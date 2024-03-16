//! Towers module.

use enum_iterator::Sequence;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::arena::grid::GridResource;
use crate::arena::GRID_SQUARE_SIZE;
use crate::collision::GameLayer;

use crate::mob::EnemyUnit;
use crate::weapons::weapon::ProjectileData;
use crate::weapons::weapon::WeaponComponent;
use crate::weapons::weapon::WeaponTypes;
use crate::weapons::weapon::WeaponUpdate;
use crate::weapons::FireWeaponEvent;
use crate::{arena::grid::GridClickEvent, assets::SpriteAssets, prelude::*};

#[derive(Debug, Copy, Clone)]
pub(crate) enum TowerLevelUpReason {
    Kill,
    Upgrade,
}
#[derive(Debug, Event, Clone, Copy)]
pub(crate) struct TowerLevelUp {
    pub(crate) entity: Entity,
    pub(crate) reason: TowerLevelUpReason,
}

use fireworks::FireWorksPlugins;

/// Tower plugin.
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tower_system)
            .add_event::<TowerLevelUp>()
            .add_systems(Update, tower_fire_system)
            .add_systems(Update, tower_level_up_on_kill)
            .add_plugins(FireWorksPlugins);
    }
}

fn tower_system(
    mut commands: Commands,
    mut grid_event: EventReader<GridClickEvent>,
    assets: Res<SpriteAssets>,
    mut grid: ResMut<GridResource>,
) {
    for event in grid_event.read() {
        match event {
            GridClickEvent::Build(weapon, transform, pos) => {
                let image = assets.tower_sprites[weapon].clone();
                let tower_component = TowerComponents {
                    tower: *weapon,
                    transform: transform.clone(),
                };
                let weapon_component: WeaponComponent = WeaponComponent::from(*weapon);

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(GRID_SQUARE_SIZE)),
                            ..Default::default()
                        },
                        transform: transform.with_scale(Vec3::splat(0.5)),
                        texture: image.clone(),

                        ..Default::default()
                    },
                    tower_component,
                    weapon_component,
                    CollisionLayers::new(GameLayer::Tower, [GameLayer::Enemy]),
                ));

                grid.set_occupied(pos, true);
            }

            _ => {}
        }
    }
}

fn tower_fire_system(
    time: Res<Time>,
    mut tower_query: Query<(Entity, &mut TowerComponents, &mut WeaponComponent)>,
    enemies_position: Query<&Transform, With<EnemyUnit>>,
    mut fire_event_writer: EventWriter<FireWeaponEvent>,
) {
    for (entity, tower, mut weapon) in tower_query.iter_mut() {
        weapon.update(time.delta());
        if let Some(mut projectile_data) = weapon.fire() {
            // Find the nearest enemy
            let tower_position = tower.transform.translation;
            let mut nearest_enemy = None;
            let mut nearest_distance = f32::MAX;

            for enemy_transform in enemies_position.iter() {
                let enemy_position = enemy_transform.translation;
                let distance = tower_position.distance(enemy_position);

                if distance < nearest_distance {
                    nearest_distance = distance;
                    nearest_enemy = Some(enemy_position);
                }
            }

            // Fire at the nearest enemy

            if let Some(nearest_enemy_position) = nearest_enemy {
                let direction = (nearest_enemy_position - tower_position).normalize();
                let velocity = (direction * projectile_data.speed_multiplier).truncate(); // Set the speed as needed
                projectile_data.source_entity = Some(entity);

                fire_event_writer.send(FireWeaponEvent {
                    weapon_projectile_data: projectile_data,
                    source_transform: tower.transform.clone(),
                    velocity: LinearVelocity(velocity),
                    source_entity: entity,
                });
            }
        }
    }
}

fn tower_level_up_on_kill(
    mut tower_level_up_event: EventReader<TowerLevelUp>,
    mut tower_query: Query<(Entity, &mut TowerComponents, &mut WeaponComponent)>,
) {
    for event in tower_level_up_event.read() {
        match event {
            TowerLevelUp {
                entity,
                reason: TowerLevelUpReason::Kill,
            } => {
                if let Ok((_tower_entity, mut tower, mut weapon)) = tower_query.get_mut(*entity) {
                    weapon.level_up();
                    tower.level_up();

                    // Here you can also update the tower and its sprite
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sequence, Copy)]
pub(crate) enum TowerTypes {
    Basic,
    Fire,
    Ice,
}
impl Display for TowerTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TowerTypes {
    pub(crate) fn set(sprite_assets: &mut SpriteAssets, asset_server: Res<AssetServer>) {
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Basic, asset_server.load("basic_tower.png"));
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Fire, asset_server.load("fire_tower.png"));
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Ice, asset_server.load("ice_tower.png"));
    }
}

#[derive(Debug, Component, Clone, Copy)]
struct TowerComponents {
    tower: TowerTypes,
    transform: Transform,
}

impl TowerComponents {
    pub(crate) fn level_up(&mut self) {}
}

impl From<TowerTypes> for WeaponTypes {
    fn from(value: TowerTypes) -> Self {
        match value {
            TowerTypes::Basic => WeaponTypes::Laser,
            TowerTypes::Fire => WeaponTypes::Fire,
            TowerTypes::Ice => WeaponTypes::Ice,
        }
    }
}

impl From<TowerTypes> for ProjectileData {
    fn from(value: TowerTypes) -> Self {
        let weapon: WeaponTypes = value.into();
        weapon.into()
    }
}

impl From<TowerTypes> for WeaponComponent {
    fn from(value: TowerTypes) -> Self {
        let weapon_type: WeaponTypes = value.into();
        weapon_type.into()
    }
}

pub mod fireworks {
    //! Fireworks module.

    use super::*;

    /// Fireworks plugin.
    pub struct FireWorksPlugins;

    impl Plugin for FireWorksPlugins {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, spawn_firework_on_level_up)
                .add_systems(Update, update_fireworks);
        }
    }

    // Firework component
    #[derive(Debug, Component)]
    struct Firework {
        lifetime: Timer,
    }

    // System to spawn fireworks
    fn spawn_firework_on_level_up(
        mut commands: Commands,
        mut tower_level_up_event: EventReader<TowerLevelUp>,
        tower_query: Query<&Transform, With<TowerComponents>>,
        assets: Res<AssetServer>,
    ) {
        for event in tower_level_up_event.read() {
            match event {
                TowerLevelUp {
                    entity,
                    reason: TowerLevelUpReason::Kill,
                } => {
                    if let Ok(transform) = tower_query.get(event.entity) {
                        // Load the firework texture as an asset
                        let firework_texture_handle = assets.load("firework.png");

                        info!("Tower level up event spawning fireworks {:?}", event);
                        commands
                            .spawn(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::splat(50.0)),
                                    ..Default::default()
                                },
                                texture: firework_texture_handle,
                                transform: Transform {
                                    translation: transform.translation + Vec3::new(0.0, 50.0, 0.0), // Offset the firework above the tower
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Firework {
                                lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                            });
                    }
                }
                _ => {}
            }
        }
    }

    // System to update fireworks
    // System to update fireworks
    fn update_fireworks(
        time: Res<Time>,
        mut commands: Commands,
        mut query: Query<(Entity, &mut Firework, &mut Transform)>,
    ) {
        let firework_speed = 50.0; // Adjust this value to change the speed of the firework

        for (entity, mut firework, mut transform) in query.iter_mut() {
            firework.lifetime.tick(time.delta());

            if firework.lifetime.finished() {
                commands.entity(entity).despawn();
            } else {
                // Move the firework upwards
                transform.translation.y += firework_speed * time.delta_seconds();
            }
        }
    }
}
