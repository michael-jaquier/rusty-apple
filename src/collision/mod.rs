//! Collisions

pub(crate) mod contact;

use bevy::{
    core_pipeline::contrast_adaptive_sharpening,
    ecs::entity,
    log::tracing_subscriber::fmt::init,
    reflect::TypeData,
    transform::commands,
    utils::{hashbrown::HashSet, info},
};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_egui::egui::debug_text::print;
use rand::{thread_rng, Rng};

use crate::{
    mob::{
        enemy::{self, ScheduledForDespawnEnemy},
        EffectType, EnemyComponent, EnemyUnit, MobDespawnEvent,
    },
    prelude::*,
    towers::{TowerComponents, TowerData, TowerInfo, TowerLevelUp, TowerLevelUpReason, TowerTypes},
    weapons::{weapon::ProjectileData, DespawnProjectileEvent, ScheduledForDespawnProjectile},
};

#[derive(PhysicsLayer)]
pub(crate) enum GameLayer {
    Projectile,
    Enemy,
    Tower,
}

/// Types of collisions
#[derive(Debug, Event)]
pub(crate) enum CollisionTypes {
    // A projectile has collided with an enemy.
    ProjectileToEnemy {
        mob_entity: Entity,
        projectile_entity: Entity,
        projectile_data: ProjectileData,
        tile: TilePos,
    },
}

/// The collision plugin.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionTypes>()
            .add_systems(Update, collision_events_types_system)
            .add_systems(Update, read_projectile_to_enemy_collision_event);
    }
}

/// The collision system

fn collision_events_types_system(
    mut collision_event_reader: EventReader<CollisionStarted>,
    enemies: Query<(Entity, &EnemyUnit, &Sprite, &TilePos)>,
    projectiles: Query<(Entity, &ProjectileData, &Sprite)>,
    mut collision_events: EventWriter<CollisionTypes>,
) {
    // Iterate through the collision events
    for collision in collision_event_reader.read() {
        // Get the entities involved in the collision
        let (entity1, entity2) = (collision.0, collision.1);
        // Get the components of the entities
        let maybe_projectile = projectiles.get(entity1);
        let maybe_enemy = enemies.get(entity2);
        if let (
            Ok((projectile_entity, projectile_data, _)),
            Ok((enemy_entity, _enemy, _, tile_pos)),
        ) = (maybe_projectile, maybe_enemy)
        {
            // Send a projectile to enemy collision event
            collision_events.send(CollisionTypes::ProjectileToEnemy {
                mob_entity: enemy_entity,
                projectile_entity,
                projectile_data: *projectile_data,
                tile: *tile_pos,
            });
        }

        // If the projectile and enemy exist, then we have a collision
    }
}

fn read_projectile_to_enemy_collision_event(
    mut enemies: Query<(Entity, &mut EnemyUnit, &mut Sprite, &Position, &TilePos)>,
    mut collision_events: EventReader<CollisionTypes>,
    mut enemy_despawn_events: EventWriter<MobDespawnEvent>,
    mut projectile_despawn_events: EventWriter<DespawnProjectileEvent>,
    mut projectile_despawn_schedule: ResMut<ScheduledForDespawnProjectile>,
    mut enemy_despawn_schedule: ResMut<ScheduledForDespawnEnemy>,
    mut tower_level_up_events: EventWriter<TowerLevelUp>,
    tower_components: Query<&TowerComponents>,
    tower_info: Res<TowerInfo>,
) {
    for event in collision_events.read() {
        match event {
            CollisionTypes::ProjectileToEnemy {
                mob_entity,
                projectile_entity,
                projectile_data,
                tile,
            } => {
                // We want to avoid the projectile doing damage to the same enemy multiple times
                // Or penetrating the enemy when we dont want it to
                if projectile_despawn_schedule.contains(projectile_entity)
                    || enemy_despawn_schedule.contains(mob_entity)
                {
                    continue;
                }

                // The the shedule it too big clean it up
                if projectile_despawn_schedule.len() > 99_000 {
                    projectile_despawn_schedule.clear();
                }

                fn tile_check(proj_tile: &TilePos, enemy_tile: &TilePos, _potency: u32) -> bool {
                    if proj_tile == enemy_tile {
                        return true;
                    }
                    false
                }

                for (entity, mut unit, _sprite, _position, enemy_tile) in enemies.iter_mut() {
                    if unit.health <= 0 {
                        if enemy_despawn_schedule.contains(&entity) {
                            continue;
                        }
                        enemy_despawn_events.send(MobDespawnEvent {
                            enemy_entity: entity,
                            spawner_id: unit.spwawner_id,
                            reason: crate::mob::EnemyDespawnReason::Killed,
                        });
                        enemy_despawn_schedule.insert(*&entity);

                        if let Some(tower_entity) = projectile_data.source_entity {
                            tower_level_up_events.send(TowerLevelUp {
                                entity: tower_entity,
                                reason: TowerLevelUpReason::Kill,
                                enemy_experience: unit.experience,
                            });
                        }
                    } else if &entity == mob_entity
                        || (projectile_data.area_of_effect && tile_check(tile, &enemy_tile, 1))
                    {
                        unit.health = unit.health.saturating_sub(projectile_data.damage);
                        if let Some(tower_entity) = projectile_data.source_entity {
                            if let Some(tower) = tower_components.get(tower_entity).ok() {
                                let tower_data = tower_info.get_data(&tower.tower);
                                let status = projectile_data.status_effect(&tower_data);
                                unit.insert_status(status);
                            }
                        }
                    }
                }

                if !projectile_despawn_schedule.contains(projectile_entity) {
                    projectile_despawn_schedule.insert(*projectile_entity);
                    projectile_despawn_events.send(DespawnProjectileEvent {
                        projectile_entity: *projectile_entity,
                    });
                }
            }
        }
    }
}
