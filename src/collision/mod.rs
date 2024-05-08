//! Collisions

pub(crate) mod contact;

use std::arch::aarch64::vld1_dup_f32;

use bevy::{core_pipeline::contrast_adaptive_sharpening, ecs::entity};
use rand::{thread_rng, Rng};

use crate::{
    mob::{
        enemy::{self, EnemyComponent, EnemyUnit},
        MobDespawnEvent,
    },
    player::PlayerUpdateEvent,
    prelude::*,
    weapon::{DespawnProjectileEvent, ProjectileData},
};

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
            .add_systems(Update, handle_collisions)
            .add_systems(Update, read_projectile_to_enemy_collision_event);
    }
}

/// The collision system

fn handle_collisions(
    mut collision_event_reader: EventReader<CollisionStarted>,
    enemies: Query<(Entity, &EnemyUnit)>,
    projectiles: Query<(Entity, &ProjectileData)>,
    mut collision_events: EventWriter<CollisionTypes>,
) {
    // Iterate through the collision events
    for collision in collision_event_reader.read() {
        // Get the entities involved in the collision
        let (entity1, entity2) = (collision.0, collision.1);
        // Get the components of the entities
        let projectile = projectiles.get(entity1);
        let enemy = enemies.get(entity2);
        // If the projectile and enemy exist, then we have a collision
        if let (Ok(projectile), Ok(enemy)) = (projectile, enemy) {
            // Dispatch the collision event
            collision_events.send(CollisionTypes::ProjectileToEnemy {
                mob_entity: entity2,
                projectile_entity: entity1,
                projectile_data: *projectile.1,
            });
        }
    }
}

fn read_projectile_to_enemy_collision_event(
    mut enemies: Query<(Entity, &mut EnemyUnit, &mut Sprite)>,
    mut collision_events: EventReader<CollisionTypes>,
    mut enemy_despawn_events: EventWriter<MobDespawnEvent>,
    mut projectile_despawn_events: EventWriter<DespawnProjectileEvent>,
    mut player_update_events: EventWriter<PlayerUpdateEvent>,
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
                            });

                            player_update_events
                                .send(PlayerUpdateEvent::add_points(unit.mob_type.points()));
                        } else {
                            unit.health = unit.health.saturating_sub(projectile_data.damage);
                        }
                    }

                    projectile_despawn_events.send(DespawnProjectileEvent {
                        projectile_entity: *projectile_entity,
                    });
                }
            }
        };
    }
}
