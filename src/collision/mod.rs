//! Collisions

pub(crate) mod contact;

use std::arch::aarch64::vld1_dup_f32;

use bevy::{
    core_pipeline::contrast_adaptive_sharpening, ecs::entity, transform::commands, utils::info,
};
use bevy_egui::egui::debug_text::print;
use rand::{thread_rng, Rng};

use crate::{
    mob::{EnemyComponent, EnemyUnit, MobDespawnEvent},
    prelude::*,
    towers::TowerLevelUp,
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
    enemies: Query<(Entity, &EnemyUnit, &Sprite)>,
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
        if let (Ok((projectile_entity, projectile_data, _)), Ok((enemy_entity, _enemy, _))) =
            (maybe_projectile, maybe_enemy)
        {
            // Send a projectile to enemy collision event
            collision_events.send(CollisionTypes::ProjectileToEnemy {
                mob_entity: enemy_entity,
                projectile_entity,
                projectile_data: *projectile_data,
            });
        }

        // If the projectile and enemy exist, then we have a collision
    }
}

fn read_projectile_to_enemy_collision_event(
    mut enemies: Query<(Entity, &mut EnemyUnit, &mut Sprite)>,
    mut collision_events: EventReader<CollisionTypes>,
    mut enemy_despawn_events: EventWriter<MobDespawnEvent>,
    mut projectile_despawn_events: EventWriter<DespawnProjectileEvent>,
    mut projectile_despawn_schedule: ResMut<ScheduledForDespawnProjectile>,
    mut tower_level_up_events: EventWriter<TowerLevelUp>,
) {
    for event in collision_events.read() {
        match event {
            CollisionTypes::ProjectileToEnemy {
                mob_entity,
                projectile_entity,
                projectile_data,
            } => {
                if let Ok((_, _, mut sprite)) = enemies.get_mut(*mob_entity) {
                    // Create a random color for the enemy
                    let color = Color::rgb(
                        thread_rng().gen_range(0.0..1.0),
                        thread_rng().gen_range(0.0..1.0),
                        thread_rng().gen_range(0.0..1.0),
                    );
                    sprite.color = color; // Change the color of the sprite
                    if let Some((entity, mut unit, _sprite)) = enemies.get_mut(*mob_entity).ok() {
                        if unit.health <= 0 {
                            enemy_despawn_events.send(MobDespawnEvent {
                                enemy_entity: entity,
                                spawner_id: unit.spwawner_id,
                                reason: crate::mob::EnemyDespawnReason::Killed,
                            });
                            if let Some(tower_entity) = projectile_data.source_entity {
                                tower_level_up_events.send(TowerLevelUp {
                                    entity: tower_entity,
                                    reason: crate::towers::TowerLevelUpReason::Kill,
                                });
                            } else {
                            }
                        } else {
                            unit.health = unit.health.saturating_sub(projectile_data.damage);
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
}
