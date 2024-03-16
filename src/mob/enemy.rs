//! Enemies

use crate::{
    arena::{
        grid::GridResource,
        path_finding::{path_mob_finding, Pos},
    },
    assets::SpriteAssets,
    collision::GameLayer,
    prelude::*,
    ui::level::{LevelUpMap, MapLevel},
};
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::TileStorage};
use enum_iterator::{all, Sequence};

use super::{
    Enemies, EnemyComponent, EnemyDespawnReason, EnemyUnit, MobDespawnEvent, MobSpawnEvent,
    MobSpawner, MobSpawnerData, SpawnId,
};

/// The mob plugin.
pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobSpawnEvent>()
            .add_event::<MobDespawnEvent>()
            .add_systems(Update, (deploy_mod_spawners, despawn_mob_spawners).chain())
            .add_systems(Update, level_up_on_kills_reached)
            .add_systems(Update, spawn_enemy)
            .add_systems(Update, mob_spawn_system)
            .add_systems(Update, follow_path)
            .add_systems(Update, (trigger_despawn_event, mob_despawn_system).chain());
    }
}

fn level_up_on_kills_reached(
    mut mob_query: Query<&EnemyComponent>,
    mut level_up_event_writer: EventWriter<LevelUpMap>,
) {
    if mob_query.iter().count() == 0 {
        return;
    }
    let f = |enemy: &EnemyComponent| enemy.spawner.current_kill >= enemy.spawner.max_kill;
    let all_killed = mob_query.iter().all(f);

    if all_killed {
        level_up_event_writer.send(LevelUpMap);
    }
}

fn mob_spawn_system(
    mut event: EventWriter<MobSpawnEvent>,
    mut mob_query: Query<&mut EnemyComponent>,
    map_level: Res<MapLevel>,
    time: Res<Time>,
) {
    for (mut enemy) in mob_query.iter_mut() {
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
    if map_level.level <= 10 && ec_count == 0 {
        let spawn_rate = (5.0 / map_level.level as f32).min(0.15);
        let timer = Timer::from_seconds(spawn_rate, TimerMode::Repeating);
        let max_count = (map_level.level as f32 * 2.0).ceil() as usize;
        let max_kill = (map_level.level as f32 * 10.0).ceil() as usize;
        // Spawn a block enemy
        let enemy = Enemies::Block;
        let component = EnemyComponent {
            mob_type: enemy,
            spawner: MobSpawner {
                mob_type: enemy,
                spawn_position: grid
                    .grid_enemy_start
                    .to_position(grid.grid_square_size, grid.bottom_left()),
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
    mut map_level_update: EventReader<LevelUpMap>,
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
) {
    for mob_spawn_event in event.read() {
        let sprite = assets.enemy_sprites[&mob_spawn_event.mob_type].clone();
        let tile = assets.other[&mob_spawn_event.mob_type].clone();
        let mut tile_storage = TileStorage::empty(TilemapTileSize { x: 16.0, y: 16.0 });
        let enemy_unit = mob_spawn_event
            .mob_type
            .into_unit(mob_spawn_event.spawner_id, mob_spawn_event.map_level);

        commands.spawn((
            SpriteBundle {
                sprite,
                transform: Transform {
                    translation: Vec3::Z * 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            mob_spawn_event.position,
            Collider::rectangle(50.0, 50.0),
            Sensor,
            RigidBody::Kinematic,
            ExternalForce::ZERO,
            LinearVelocity::ZERO,
            AngularVelocity(0.0),
            CollisionLayers::new(GameLayer::Enemy, [GameLayer::Projectile, GameLayer::Tower]),
            enemy_unit,
        ));
    }
}

fn mob_despawn_system(
    mut commands: Commands,
    mut event: EventReader<MobDespawnEvent>,
    mut query: Query<&mut EnemyComponent>,
) {
    for mob_despawn_event in event.read() {
        let maybe_entity = commands.get_entity(mob_despawn_event.enemy_entity);
        if let Some(mut entity) = maybe_entity {
            // get the enemy component that matches our id
            for mut enemy in query.iter_mut() {
                if enemy.spawner.spawner_id == mob_despawn_event.spawner_id {
                    enemy.spawner.current_count = enemy.spawner.current_count.saturating_sub(1);
                    match mob_despawn_event.reason {
                        EnemyDespawnReason::Killed => {
                            enemy.spawner.current_kill += 1;
                        }
                        _ => {}
                    }
                }
            }

            entity.despawn()
        }
    }
}

fn trigger_despawn_event(
    mut event: EventWriter<MobDespawnEvent>,
    query: Query<(Entity, &EnemyUnit, &Position)>,
    grid: Res<GridResource>,
) {
    // Count the number of enemies in the arena
    let count = query.iter().count();
    if count > 10 {
        for (entity, enemy, position) in query.iter() {
            let grid_x_max = grid.top_left().0;
            let grid_y_max = grid.top_left().1;

            if position.x.abs() > grid_x_max as f32 || position.y.abs() > grid_y_max as f32 {
                event.send(MobDespawnEvent {
                    enemy_entity: entity,
                    spawner_id: enemy.spwawner_id,
                    reason: EnemyDespawnReason::ReachedEnd,
                });
            }
        }
    }
}

fn follow_path(
    time: Res<Time>,
    grid: Res<GridResource>,
    mut query: Query<(Entity, &mut Transform, &mut EnemyUnit), With<EnemyUnit>>,
    mut mob_despawn_event: EventWriter<MobDespawnEvent>,
) {
    for (entity, mut transform, mut enemy_unit) in query.iter_mut() {
        if enemy_unit.move_timer.tick(time.delta()).just_finished() {
            let current =
                Pos::from_transform(&transform, grid.grid_square_size, grid.bottom_left());
            let path = path_mob_finding(&grid, current);

            if let Some(next_pos) = path {
                let next_transform =
                    next_pos.to_transform(grid.grid_square_size, grid.bottom_left());

                // Move the block instantly to the new position
                transform.translation = next_transform.translation;
            } else {
                // If the enemy unit has reached the end of the path, despawn it
                mob_despawn_event.send(MobDespawnEvent {
                    enemy_entity: entity,
                    spawner_id: enemy_unit.spwawner_id,
                    reason: EnemyDespawnReason::ReachedEnd,
                });
            }

            // Reset the move timer
            enemy_unit.move_timer.reset();
        }
    }
}
