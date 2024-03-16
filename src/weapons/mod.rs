//! Weapons
use std::collections::HashSet;

use crate::prelude::*;

use self::weapon::{despawn_project_system, weapon_fire_system, ProjectileData, WeaponTypes};
pub(crate) mod weapon;

/// This system will despawn the laser when the timer runs out
pub struct WeaponPlugin;

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

#[derive(Debug, Default, Resource)]
pub(crate) struct ScheduledForDespawnProjectile(HashSet<Entity>);
impl ScheduledForDespawnProjectile {
    pub(crate) fn insert(&mut self, entity: Entity) {
        self.0.insert(entity);
    }
    pub(crate) fn contains(&self, entity: &Entity) -> bool {
        self.0.contains(entity)
    }
    pub(crate) fn remove(&mut self, entity: &Entity) {
        self.0.remove(entity);
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
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

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireWeaponEvent>()
            .add_event::<DespawnProjectileEvent>()
            .insert_resource(ScheduledForDespawnProjectile::default())
            .add_event::<WeaponUpgradeEvent>()
            .add_systems(Update, weapon_fire_system)
            .add_systems(Update, despawn_project_system);
    }
}
