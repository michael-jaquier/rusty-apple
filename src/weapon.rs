//! This module contains the weapon system
use std::{sync::WaitTimeoutResult, time::Duration};

use bevy::time;

use crate::{assets::SpriteAssets, prelude::*};
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]

pub(crate) enum WeaponTypes {
    Laser,
}

impl From<WeaponTypes> for WeaponComponent {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => WeaponComponent {
                projectile_data: ProjectileData {
                    count: 1,
                    damage: 1,
                    weapon_type: WeaponTypes::Laser,
                },
                reload_timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        }
    }
}

/// This system will despawn the laser when the timer runs out

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireWeaponEvent>()
            .add_systems(Update, weapon_fire_system)
            .add_systems(Update, weapon_update_system);
    }
}

/// The weapon update system
fn weapon_update_system(
    mut commands: Commands,
    time: Res<time::Time>,
    assets: Res<SpriteAssets>,
    mut weapon_query: Query<(Entity, &mut WeaponComponent, &Transform)>,
) {
    for (entity, mut weapon_component, transform) in weapon_query.iter_mut() {
        if let Some(projectile_data) = weapon_component.update(time.delta()) {
            let weapon = FireWeaponEvent {
                weapon_projectile_data: projectile_data,
                source_transform: transform.clone(),
                source_entity: entity,
                velocity: LinearVelocity(Vec2::new(0.0, 100.0)),
            };
            let source_transform = weapon.source_transform.clone();
            let velocity = weapon.velocity.clone();

            commands.spawn((
                SpriteBundle {
                    sprite: assets.weapon_sprites[&projectile_data.weapon_type].clone(),
                    transform: Transform {
                        translation: source_transform.translation,
                        rotation: source_transform.rotation,
                        scale: source_transform.scale,
                    },
                    ..Default::default()
                },
                LinearVelocity(*velocity),
                RigidBody::Dynamic,
                Collider::cuboid(10.0, 10.0),
                ExternalForce::ZERO,
                projectile_data.clone(),
                Sensor,
            ));
        }
    }
}

fn weapon_fire_system(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut fire_weapon_events: EventReader<FireWeaponEvent>,
) {
    for weapon in fire_weapon_events.read() {
        let source_transform = weapon.source_transform.clone();
        let velocity = weapon.velocity.clone();
        commands.spawn((
            SpriteBundle {
                sprite: assets.weapon_sprites[&weapon.weapon_projectile_data.weapon_type].clone(),
                transform: Transform {
                    translation: source_transform.translation,
                    rotation: source_transform.rotation,
                    scale: source_transform.scale,
                },
                ..Default::default()
            },
            LinearVelocity(*velocity),
            RigidBody::Kinematic,
            Collider::cuboid(10.0, 10.0),
            ExternalForce::ZERO,
            weapon.weapon_projectile_data,
            Sensor,
        ));
    }
}
#[derive(Debug, Clone, Component, Copy)]
pub(crate) struct ProjectileData {
    pub count: usize,
    pub damage: usize,
    pub weapon_type: WeaponTypes,
}

#[derive(Debug, Clone, Component)]
pub(crate) struct WeaponComponent {
    pub projectile_data: ProjectileData,
    pub reload_timer: Timer,
}

impl WeaponComponent {
    pub(crate) fn fire(&mut self) -> Option<ProjectileData> {
        if self.can_fire() {
            self.reload_timer.reset();
            Some(self.projectile_data.clone())
        } else {
            None
        }
    }

    pub(crate) fn can_fire(&self) -> bool {
        self.reload_timer.finished()
    }
}

trait WeaponUpdate {
    fn update(&mut self, time: Duration) -> Option<ProjectileData>;
}

impl WeaponUpdate for WeaponComponent {
    fn update(&mut self, time: Duration) -> Option<ProjectileData> {
        self.reload_timer.tick(time);
        if self.can_fire() {
            None
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Event)]
pub(crate) struct FireWeaponEvent {
    /// The projectile data for the weapon.
    pub weapon_projectile_data: ProjectileData,
    /// The transform of the source entity.
    pub source_transform: Transform,
    /// The entity that is the source of the projectile.
    pub source_entity: Entity,
    /// The velocity of the source entity.
    pub velocity: LinearVelocity,
}
