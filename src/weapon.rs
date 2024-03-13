//! This module contains the weapon system
use std::hash::Hash;
use std::time::Duration;

use crate::{assets::SpriteAssets, prelude::*};
use bevy::time;
use enum_iterator::Sequence;
use std::fmt::Display;
use std::fmt::Formatter;
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Sequence)]
pub(crate) enum WeaponTypes {
    Laser,
    Torpedo,
}

impl Display for WeaponTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl WeaponTypes {
    pub(crate) fn set(assets: &mut SpriteAssets) {
        let sprites = [
            (WeaponTypes::Laser, laser_weapon_sprite()),
            (WeaponTypes::Torpedo, torpedo_weapon_sprite()),
        ];
        assets.weapon_sprites.extend(sprites.iter().cloned());
    }
}
fn laser_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::rgb(1.0, 0.0, 0.0),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..Default::default()
    }
}

fn torpedo_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::rgb(0.0, 0.0, 1.0),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..Default::default()
    }
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
                reload_timer: Timer::from_seconds(1.3, TimerMode::Once),
                cost: 0,
            },
            WeaponTypes::Torpedo => WeaponComponent {
                projectile_data: ProjectileData {
                    count: 2,
                    damage: 2,
                    weapon_type: WeaponTypes::Torpedo,
                },
                reload_timer: Timer::from_seconds(1.0, TimerMode::Once),
                cost: 0,
            },
        }
    }
}

/// This system will despawn the laser when the timer runs out

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireWeaponEvent>()
            .add_event::<DespawnProjectileEvent>()
            .add_event::<WeaponUpgradeEvent>()
            .add_systems(Update, weapon_fire_system)
            .add_systems(Update, weapon_update_system)
            .add_systems(Update, despawn_project_system);
    }
}

/// The weapon update system
fn weapon_update_system(
    mut commands: Commands,
    time: Res<time::Time>,
    assets: Res<SpriteAssets>,
    mut weapon_query: Query<(Entity, &mut WeaponComponent, &Transform)>,
) {
    for (entity, mut weapon, transform) in weapon_query.iter_mut() {
        println!("Weapon: {:?}", weapon);
        if let Some(projectile_data) = weapon.update(time.delta()) {
            let weapon = FireWeaponEvent {
                weapon_projectile_data: projectile_data,
                source_transform: transform.clone(),
                source_entity: entity,
                velocity: LinearVelocity(Vec2::new(0.0, 300.0)),
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
                Collider::rectangle(10.0, 10.0),
                ExternalForce::ZERO,
                projectile_data.clone(),
            ));
        }
    }
}

fn despawn_project_system(
    mut commands: Commands,
    mut despawn_projectile_events: EventReader<DespawnProjectileEvent>,
) {
    for despawn_projectile_event in despawn_projectile_events.read() {
        if let Some(mut entity) = commands.get_entity(despawn_projectile_event.projectile_entity) {
            entity.despawn();
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
            Collider::rectangle(10.0, 10.0),
            ExternalForce::ZERO,
            Mass(2000.0),
            weapon.weapon_projectile_data,
        ));
    }
}
#[derive(Debug, Clone, Component, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectileData {
    pub count: usize,
    pub damage: usize,
    pub weapon_type: WeaponTypes,
}

#[derive(Debug, Clone, Component, PartialEq, Eq)]
pub(crate) struct WeaponComponent {
    pub projectile_data: ProjectileData,
    pub reload_timer: Timer,
    pub cost: usize,
}

impl Hash for WeaponComponent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.projectile_data.weapon_type.hash(state);
    }
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
            Some(self.projectile_data)
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

#[derive(Debug, Clone, Event)]
pub(crate) struct DespawnProjectileEvent {
    pub(crate) projectile_entity: Entity,
}

#[derive(Debug, Clone, Event)]
pub(crate) struct WeaponUpgradeEvent {
    pub(crate) weapon: WeaponTypes,
    pub(crate) player: Entity,
}
