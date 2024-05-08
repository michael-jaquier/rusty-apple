//! This module contains the weapon system
use std::hash::Hash;
use std::time::Duration;

use crate::arena::grid::GridResource;
use crate::collision::GameLayer;
use crate::mob::{EffectType, StatusEffect};
use crate::towers::{TowerData, TowerInfo};
use crate::{assets::SpriteAssets, prelude::*};
use bevy::log::tracing_subscriber::fmt::format::Format;
use enum_iterator::Sequence;
use rand::Rng;
use std::fmt::Display;
use std::fmt::Formatter;

use super::{DespawnProjectileEvent, FireWeaponEvent, ScheduledForDespawnProjectile};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Sequence)]
pub(crate) enum WeaponTypes {
    Laser,
    Fire,
    Ice,
    Rifle,
}

#[derive(Debug, Component)]
pub(crate) struct DespawnTimer(Timer);

impl From<WeaponTypes> for EffectType {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => EffectType::None,
            WeaponTypes::Fire => EffectType::None,
            WeaponTypes::Ice => EffectType::Slow,
            WeaponTypes::Rifle => EffectType::None,
        }
    }
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
            (WeaponTypes::Rifle, rifle_weapon_spirte()),
        ];
        assets.weapon_sprites.extend(sprites.iter().cloned());
    }

    pub(crate) fn range(&self, grid: &GridResource, level_mutator: u32) -> f32 {
        let base = match self {
            WeaponTypes::Laser => 4.5,
            WeaponTypes::Fire => 2.5,
            WeaponTypes::Ice => 4.0,
            WeaponTypes::Rifle => 10.,
        };

        base * grid.grid_square_size
    }
}

impl From<WeaponTypes> for Sprite {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => laser_weapon_sprite(),
            WeaponTypes::Fire => fire_weapon_sprite(),
            WeaponTypes::Ice => ice_weapon_sprite(),
            WeaponTypes::Rifle => rifle_weapon_spirte(),
        }
    }
}

fn laser_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::CRIMSON,
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..Default::default()
    }
}

fn fire_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::ORANGE_RED,
        custom_size: Some(Vec2::new(16.0, 16.0)),
        ..Default::default()
    }
}

fn ice_weapon_sprite() -> Sprite {
    Sprite {
        color: Color::ALICE_BLUE,
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..Default::default()
    }
}

fn rifle_weapon_spirte() -> Sprite {
    Sprite {
        color: Color::ANTIQUE_WHITE,
        custom_size: Some(Vec2::new(6.0, 6.0)),
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
                area_of_effect: false,
                source_entity: None,
            },
            WeaponTypes::Fire => ProjectileData {
                count: 1,
                damage: 20,
                weapon_type: WeaponTypes::Fire,
                speed_multiplier: 100.0,
                collision_size: (size.x, size.y),
                area_of_effect: true,
                source_entity: None,
            },

            WeaponTypes::Ice => ProjectileData {
                count: 4,
                damage: 1,
                weapon_type: WeaponTypes::Ice,
                speed_multiplier: 200.0,
                collision_size: (size.x, size.y),
                area_of_effect: false,
                source_entity: None,
            },
            WeaponTypes::Rifle => ProjectileData {
                count: 1,
                damage: 30,
                weapon_type: WeaponTypes::Rifle,
                speed_multiplier: 1600.0,
                collision_size: (size.x, size.y),
                area_of_effect: false,
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
            WeaponTypes::Rifle => Timer::from_seconds(4.9, TimerMode::Once),
        }
    }
}

impl From<WeaponTypes> for WeaponComponent {
    fn from(value: WeaponTypes) -> Self {
        match value {
            WeaponTypes::Laser => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                level: 1,
            },
            WeaponTypes::Fire => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                level: 1,
            },
            WeaponTypes::Ice => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                level: 1,
            },
            WeaponTypes::Rifle => WeaponComponent {
                projectile_data: ProjectileData::from(value),
                reload_timer: Timer::from(value),
                level: 1,
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

pub(crate) fn despawn_timer_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).despawn();
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
            weapon.weapon_projectile_data,
            CollisionLayers::new(GameLayer::Projectile, [GameLayer::Enemy]),
            DespawnTimer(Timer::from_seconds(5.0, TimerMode::Once)),
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
    pub area_of_effect: bool,
    pub source_entity: Option<Entity>,
}

impl Display for ProjectileData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Proj Count: {:?}\nDamage: {}\n Speed: {}\n AoE: {}",
            self.count, self.damage, self.speed_multiplier, self.area_of_effect
        )
    }
}

impl ProjectileData {
    pub(crate) fn status_effect(&self, tower: &TowerData) -> StatusEffect {
        let (potency, duration) = (tower.get_status().potency, tower.get_status().duration);
        StatusEffect {
            effect_type: self.weapon_type.into(),
            timer: Timer::from_seconds(duration as f32, TimerMode::Once),
            potency: potency,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub(crate) struct WeaponComponent {
    pub projectile_data: ProjectileData,
    pub reload_timer: Timer,
    pub level: u32,
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
        let (min, max, flat) = match self.projectile_data.weapon_type {
            WeaponTypes::Laser => (0.01, 0.10, 15),
            WeaponTypes::Fire => (0.01, 0.10, 20),
            WeaponTypes::Ice => (0.01, 0.03, 10),
            WeaponTypes::Rifle => (0.01, 0.23, 45),
        };

        let mut dmg_rng = rand::thread_rng();

        let dmg_boost = (self.projectile_data.damage as f32 * dmg_rng.gen_range(min..max))
            + dmg_rng.gen_range(0..flat) as f32;
        self.projectile_data.damage = (self.projectile_data.damage as f32 + dmg_boost) as usize;

        if self.level % 5 == 0 {
            self.projectile_data.speed_multiplier =
                (self.projectile_data.speed_multiplier + 50.).min(2000.0);
        }

        if self.level % 25 == 0 {
            self.projectile_data.count = self.projectile_data.count + 1;
        }

        if self.level % 10 == 0 {
            self.reload_timer = Timer::from_seconds(
                (self.reload_timer.duration().as_secs_f32() * 0.9).max(0.4),
                TimerMode::Once,
            );
        }

        if self.level > 100000 {
            self.projectile_data.area_of_effect = true;
        }

        self.level += 1;
    }

    pub(crate) fn cost(&self) -> u32 {
        let base = match self.projectile_data.weapon_type {
            WeaponTypes::Laser => 10,
            WeaponTypes::Fire => 15,
            WeaponTypes::Ice => 20,
            WeaponTypes::Rifle => 25,
        };
        base * self.level
    }

    pub(crate) fn weapon_type(&self) -> WeaponTypes {
        self.projectile_data.weapon_type
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

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use enum_iterator::all;

    use crate::towers::TowerTypes;

    use super::*;

    fn calculate_weapon_points(
        cost: u32,
        weapon: &WeaponComponent,
        grid: &GridResource,
        level: u32,
    ) -> f32 {
        let range_points = weapon.weapon_type().range(&grid, level) * 3.0;
        let count = (weapon.projectile_data.count as f32 / 1.5).max(1.0);
        let damage_points = weapon.projectile_data.damage as f32;
        let reload_time_points = 1.0 / (weapon.reload_timer.duration().as_secs_f32() * 2.0);
        let speed_multiplier_points = weapon.projectile_data.speed_multiplier / 300.0;

        let effect_type_points = match EffectType::from(weapon.weapon_type()) {
            EffectType::None => 1.0,
            EffectType::Slow => 2.5,
            // Add more cases as needed
        };
        let aoe_scaling = if weapon.projectile_data.area_of_effect {
            2.0
        } else {
            1.0
        };

        let formula = speed_multiplier_points
            + ((count * effect_type_points * (aoe_scaling * (range_points + damage_points)))
                / reload_time_points);
        let cost_scaling = formula / cost as f32;
        cost_scaling
    }

    #[test]
    fn test_weapon_balance() {
        let grid = GridResource::default();
        let level = 1;
        let mut points_per_weapon = HashMap::new();

        for tower in all::<TowerTypes>() {
            let cost = tower.cost(10);
            let weapon_type: WeaponTypes = tower.into();
            let mut weapon = WeaponComponent::from(weapon_type);
            for _ in 0..10 {
                weapon.level_up();
            }

            let points = calculate_weapon_points(cost, &weapon, &grid, level);
            points_per_weapon.insert(weapon_type, points);
        }
        // Ensure all weapons are within 130% of each other
        let mut points = points_per_weapon.values().cloned().collect::<Vec<f32>>();
        points.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let max = points.last().unwrap();
        let min = points.first().unwrap();
        let diff = max - min;
        let max_diff = diff / min;
        println!("Points: {:?}", points_per_weapon);
        assert!(
            max_diff < 1.5,
            "Weapons are not balanced {:?},
            Diff: {:?}",
            points_per_weapon,
            max_diff
        );
    }
}
