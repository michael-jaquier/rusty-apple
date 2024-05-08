//! Enemies

use std::collections::HashSet;

use bevy_ecs_tilemap::tiles::TilePos;

use crate::{
    arena::{
        grid::GridResource,
        path_finding::{from_transform, path_mob_finding, to_position, to_transform},
    },
    assets::SpriteAssets,
    collision::GameLayer,
    player::PlayerUpdateEvent,
    prelude::*,
    ui::level::{LevelMap, MapLevel},
};

use super::{
    EffectType, Enemies, EnemyComponent, EnemyDespawnReason, EnemyUnit, MobDespawnEvent,
    MobSpawnEvent, MobSpawner, SpawnId,
};

/// The mob plugin.
pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobSpawnEvent>()
            .add_event::<MobDespawnEvent>()
            .insert_resource(ScheduledForDespawnEnemy::default())
            .add_systems(Update, (deploy_mod_spawners, despawn_mob_spawners).chain())
            .add_systems(Update, level_up_on_kills_reached)
            .add_systems(Update, spawn_enemy)
            .add_systems(Update, mob_spawn_system)
            .add_systems(Update, follow_path)
            .add_systems(
                Update,
                (trigger_move_to_start_position, mob_despawn_system).chain(),
            );
    }
}

fn level_up_on_kills_reached(
    mob_query: Query<&EnemyComponent>,
    mut level_up_event_writer: EventWriter<LevelMap>,
) {
    if mob_query.iter().count() == 0 {
        return;
    }
    let f = |enemy: &EnemyComponent| enemy.spawner.current_kill >= enemy.spawner.max_kill;
    let all_killed = mob_query.iter().all(f);

    if all_killed {
        level_up_event_writer.send(LevelMap::LevelUp(1));
    }
}

fn mob_spawn_system(
    mut event: EventWriter<MobSpawnEvent>,
    mut mob_query: Query<&mut EnemyComponent>,
    map_level: Res<MapLevel>,
    time: Res<Time>,
) {
    for mut enemy in mob_query.iter_mut() {
        if enemy.spawner.timer.tick(time.delta()).just_finished()
            && enemy.spawner.current_count < enemy.spawner.max_count
        {
            event.send(MobSpawnEvent {
                mob_type: enemy.mob_type,
                position: enemy.spawner.spawn_position,
                spawner_id: enemy.spawner.spawner_id,
                map_level: map_level.level,
            });
            enemy.spawner.current_count += 1;
            enemy.spawner.timer.reset();
        }
    }
}

fn deploy_mod_spawners(
    mut commands: Commands,
    ec: Query<&mut EnemyComponent>,
    map_level: Res<MapLevel>,
    grid: Res<GridResource>,
) {
    let ec_count = ec.iter().count();
    if ec_count == 0 {
        let spawn_rate = (5.0 / map_level.level as f32).min(0.15);
        let timer = Timer::from_seconds(spawn_rate, TimerMode::Repeating);
        let max_count = (map_level.level as f32 * 2.0).ceil() as usize;
        let max_kill = (map_level.level as f32 * 10.0).ceil() as usize;
        // Spawn a block enemy
        let enemy = Enemies::Block;
        let postion = to_position(grid.grid_enemy_start, &grid);
        let component = EnemyComponent {
            mob_type: enemy,
            spawner: MobSpawner {
                mob_type: enemy,
                spawn_position: postion,
                timer,
                max_count,
                current_count: 0,
                max_kill,
                current_kill: 0,
                spawner_id: SpawnId::new(),
            },
        };
        let mut entity = commands.spawn_empty();
        entity.insert(component);
    }
}

fn despawn_mob_spawners(
    mut commands: Commands,
    mut ec: Query<(Entity, &mut EnemyComponent)>,
    mut map_level_update: EventReader<LevelMap>,
) {
    for _ in map_level_update.read() {
        for (entity, _ec) in ec.iter_mut() {
            let maybe_ec = commands.get_entity(entity);
            if let Some(mut entity) = maybe_ec {
                entity.despawn()
            }
        }
    }
}

fn spawn_enemy(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut event: EventReader<MobSpawnEvent>,
    grid: Res<GridResource>,
) {
    for mob_spawn_event in event.read() {
        let sprite = assets.enemy_sprites[&mob_spawn_event.mob_type].clone();

        let enemy_unit = mob_spawn_event
            .mob_type
            .into_unit(mob_spawn_event.spawner_id, mob_spawn_event.map_level);
        let init_transform = to_position(grid.grid_enemy_start, &grid);

        commands.spawn((
            SpriteBundle {
                sprite,
                transform: Transform {
                    translation: Vec3::Z * 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            init_transform,
            Collider::rectangle(16.0, 16.0),
            Sensor,
            RigidBody::Kinematic,
            ExternalForce::ZERO,
            LinearVelocity::ZERO,
            AngularVelocity(0.0),
            CollisionLayers::new(GameLayer::Enemy, [GameLayer::Projectile, GameLayer::Tower]),
            enemy_unit,
            grid.grid_enemy_start,
        ));
    }
}

fn mob_despawn_system(
    mut commands: Commands,
    mut event: EventReader<MobDespawnEvent>,
    mut query: Query<&mut EnemyComponent>,
    mut enemy_schedule: ResMut<ScheduledForDespawnEnemy>,
    mut player: EventWriter<PlayerUpdateEvent>,
) {
    for mob_despawn_event in event.read() {
        if let Some(mut entity) = commands.get_entity(mob_despawn_event.enemy_entity) {
            entity.despawn();
            enemy_schedule.remove(&mob_despawn_event.enemy_entity);
            // get the enemy component that matches our id
            for mut enemy in query.iter_mut() {
                if enemy.spawner.spawner_id == mob_despawn_event.spawner_id {
                    enemy.spawner.current_count = enemy.spawner.current_count.saturating_sub(1);
                    match mob_despawn_event.reason {
                        EnemyDespawnReason::Killed => {
                            enemy.spawner.current_kill += 1;
                            player.send(PlayerUpdateEvent::Bricks(1));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn trigger_move_to_start_position(
    mut query: Query<(Entity, &EnemyUnit, &mut Position, &mut TilePos)>,
    grid: Res<GridResource>,
    enemy_schedule: ResMut<ScheduledForDespawnEnemy>,
    mut player_update_event: EventWriter<PlayerUpdateEvent>,
    map_level: Res<MapLevel>,
) {
    // Count the number of enemies in the arena
    for (entity, enemy, mut position, mut tile_pos) in query.iter_mut() {
        if enemy_schedule.contains(&entity) {
            continue;
        }

        let end_position = to_position(grid.grid_enemy_end, &grid);

        if *position == end_position {
            let start_position = to_position(grid.grid_enemy_start, &grid);
            *position = start_position;
            *tile_pos = grid.grid_enemy_start;
            player_update_event.send(PlayerUpdateEvent::Damage(
                enemy.mob_type.damage(map_level.level),
            ));
        }
    }
}

fn follow_path(
    time: Res<Time>,
    grid: Res<GridResource>,
    mut query: Query<(Entity, &mut Transform, &mut EnemyUnit, &mut TilePos), With<EnemyUnit>>,
    mut mob_despawn_event: EventWriter<MobDespawnEvent>,
    mut enemy_schedule: ResMut<ScheduledForDespawnEnemy>,
) {
    for (entity, mut transform, mut enemy_unit, mut tile_pos) in query.iter_mut() {
        if enemy_unit
            .next_position
            .map_or(true, |next| transform.translation.distance(next) < 0.1)
        {
            let current = from_transform(&transform, grid.grid_square_size, grid.bottom_left());
            let path = path_mob_finding(&grid, current);
            if let Some(next_pos) = path {
                enemy_unit.next_position = Some(to_transform(next_pos, &grid).translation);
                *tile_pos = next_pos;
            } else {
                // If the enemy unit has reached the end of the path, despawn it
                if enemy_schedule.contains(&entity) {
                    continue;
                }
                mob_despawn_event.send(MobDespawnEvent {
                    enemy_entity: entity,
                    spawner_id: enemy_unit.spwawner_id,
                    reason: EnemyDespawnReason::ReachedEnd,
                });
                enemy_schedule.insert(entity);
                continue;
            }
        }

        // Move the enemy unit towards the next position
        if let Some(next_position) = enemy_unit.next_position {
            let mut speed = enemy_unit.move_speed;
            if let Some(slow) = enemy_unit.status_effects.get_mut(&EffectType::Slow) {
                slow.timer.tick(time.delta());
                if slow.timer.just_finished() {
                    enemy_unit.status_effects.remove(&EffectType::Slow);
                } else {
                    let slow_effect = (100u32.saturating_sub(slow.potency)).max(1) as f32 / 100.0;
                    speed *= slow_effect;
                }
            }

            let distance_to_target = transform.translation.distance(next_position);
            let distance_to_move = speed * time.delta_seconds();

            if distance_to_target < distance_to_move {
                // If the entity would move past the target position, check for the next target position
                let current = from_transform(&transform, grid.grid_square_size, grid.bottom_left());
                let path = path_mob_finding(&grid, current);
                if let Some(next_pos) = path {
                    // If the next target position is available, start moving towards it
                    enemy_unit.next_position = Some(to_transform(next_pos, &grid).translation);
                    let direction = (next_position - transform.translation).normalize();
                    transform.translation += direction * distance_to_move;
                } else {
                    // If the next target position is not available, set the entity's position to the current target position directly
                    transform.translation = next_position;
                }
            } else {
                let direction = (next_position - transform.translation).normalize();
                transform.translation += direction * distance_to_move;
            }
        }
    }
}
#[derive(Debug, Resource, Deref, Default, DerefMut)]
pub(crate) struct ScheduledForDespawnEnemy(pub(crate) HashSet<Entity>);
