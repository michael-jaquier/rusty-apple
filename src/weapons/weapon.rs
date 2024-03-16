//! This module contains the weapon system
use std::hash::Hash;
use std::time::Duration;

use crate::collision::GameLayer;
use crate::{assets::SpriteAssets, prelude::*};
use enum_iterator::Sequence;
use std::fmt::Display;
use std::fmt::Formatter;

use super::{DespawnProjectileEvent, FireWeaponEvent, ScheduledForDespawnProjectile};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Sequence)]
pub(crate) enum WeaponTypes {
    Laser,
    Fire,
    Ice,
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
            (WeaponTypes::Fire, fire_weapon_sprite()),
            (WeaponTypes::Ice, ice_weapon_sprite()),
        ];
        assets.weapon_sprites.extend(sprites.iter().cloned());
    }
}

impl From<WeaponTypes> for Sprite {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => laser_weapon_sprite(),
            WeaponTypes::Fire => fire_weapon_sprite(),
            WeaponTypes::Ice => ice_weapon_sprite(),
        }
    }
}

fn laser_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::rgb(1.0, 0.0, 0.0),
        custom_size: Some(Vec2::new(30.0, 3.0)),
        ..Default::default()
    }
}

fn fire_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::ORANGE_RED,
        custom_size: Some(Vec2::new(36.0, 36.0)),
        ..Default::default()
    }
}

fn ice_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::ALICE_BLUE,
        custom_size: Some(Vec2::new(20.0, 3.0)),
        ..Default::default()
    }
}

impl From<WeaponTypes> for ProjectileData {
    fn from(value: WeaponTypes) -> Self {
        let sprite: Sprite = value.into();
        let size = sprite.custom_size.unwrap();
        match value {
            WeaponTypes::Laser => ProjectileData {
                count: 1,
                damage: 3,
                weapon_type: WeaponTypes::Laser,
                speed_multiplier: 900.0,
                collision_size: (size.x, size.y),
                source_entity: None,
            },
            WeaponTypes::Fire => ProjectileData {
                count: 1,
                damage: 10,
                weapon_type: WeaponTypes::Fire,
                speed_multiplier: 100.0,
                collision_size: (size.x, size.y),
                source_entity: None,
            },

            WeaponTypes::Ice => ProjectileData {
                count: 4,
                damage: 1,
                weapon_type: WeaponTypes::Ice,
                speed_multiplier: 200.0,
                collision_size: (size.x, size.y),
                source_entity: None,
            },
        }
    }
}

impl From<WeaponTypes> for Timer {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => Timer::from_seconds(1.3, TimerMode::Once),
            WeaponTypes::Fire => Timer::from_seconds(3.3, TimerMode::Once),
            WeaponTypes::Ice => Timer::from_seconds(2.3, TimerMode::Once),
        }
    }
}

impl From<WeaponTypes> for WeaponComponent {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                cost: 0,
            },
            WeaponTypes::Fire => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                cost: 0,
            },
            WeaponTypes::Ice => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                cost: 0,
            },
        }
    }
}

pub(crate) fn despawn_project_system(
    mut commands: Commands,
    mut despawn_projectile_events: EventReader<DespawnProjectileEvent>,
    mut despawn_projectule_schedule: ResMut<ScheduledForDespawnProjectile>,
) {
    for despawn_projectile_event in despawn_projectile_events.read() {
        if let Some(mut entity) = commands.get_entity(despawn_projectile_event.projectile_entity) {
            entity.despawn();
            despawn_projectule_schedule.remove(&despawn_projectile_event.projectile_entity);
        }
    }
}

pub(crate) fn weapon_fire_system(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut fire_weapon_events: EventReader<FireWeaponEvent>,
) {
    for weapon in fire_weapon_events.read() {
        let source_transform = weapon.source_transform.clone();
        let velocity = weapon.velocity.clone();
        // Ensure the collider is slightly larger than the sprite
        let colider = Collider::rectangle(
            weapon.weapon_projectile_data.collision_size.0 * 1.5,
            weapon.weapon_projectile_data.collision_size.1 * 1.5,
        );

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
            colider,
            ExternalForce::ZERO,
            Mass(2000.0),
            weapon.weapon_projectile_data,
            CollisionLayers::new(GameLayer::Projectile, [GameLayer::Enemy]),
        ));
    }
}
#[derive(Debug, Clone, Component, Copy, PartialEq)]
pub(crate) struct ProjectileData {
    pub count: usize,
    pub damage: usize,
    pub weapon_type: WeaponTypes,
    pub speed_multiplier: f32,
    pub collision_size: (f32, f32),
    pub source_entity: Option<Entity>,
}

#[derive(Debug, Clone, Component)]
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

    pub(crate) fn level_up(&mut self) {
        self.projectile_data.damage *= 2;
    }
}

pub(crate) trait WeaponUpdate {
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
