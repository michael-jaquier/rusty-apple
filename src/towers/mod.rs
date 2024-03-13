//! Towers module.

use enum_iterator::Sequence;
use std::fmt::Display;
use std::fmt::Formatter;
use std::time::Duration;

use crate::arena::grid;
use crate::arena::GRID_SQUARE_SIZE;
use crate::assets;
use crate::weapon::WeaponComponent;
use crate::weapon::WeaponTypes;
use crate::{arena::grid::GridClickEvent, assets::SpriteAssets, prelude::*};

/// Tower plugin.
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tower_system)
            .add_systems(Update, tower_fire_system);
    }
}

fn tower_system(
    mut commands: Commands,
    mut grid_event: EventReader<GridClickEvent>,
    assets: Res<SpriteAssets>,
) {
    for event in grid_event.read() {
        match event {
            GridClickEvent::Build(weapon, transform) => {
                let image = assets.tower_sprites[weapon].clone();
                let proj_test: WeaponComponent = WeaponTypes::Laser.into();
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
                    TowerComponents {
                        tower: *weapon,
                        transform: transform.clone(),
                        reload_timer: Timer::from_seconds(1.0, TimerMode::Once),
                    },
                    proj_test,
                ));
            }

            _ => {}
        }
    }
}

fn tower_fire_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tower_query: Query<&mut TowerComponents>,
    assets: Res<SpriteAssets>,
) {
    for mut tower in tower_query.iter_mut() {
        tower.update(time.delta());
        if tower.fire() {
            commands.spawn((
                SpriteBundle {
                    sprite: assets.weapon_sprites[&WeaponTypes::Laser].clone(),
                    transform: tower.transform.clone(),
                    ..Default::default()
                },
                LinearVelocity(Vec2::new(300.0, 000.0)),
                RigidBody::Kinematic,
                Collider::rectangle(10.0, 10.0),
                ExternalForce::ZERO,
                ExternalImpulse::new(Vec2::X),
                Mass(1.0),
            ));
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

#[derive(Debug, Component)]
struct TowerComponents {
    tower: TowerTypes,
    transform: Transform,
    reload_timer: Timer,
}

impl TowerComponents {
    pub(crate) fn fire(&mut self) -> bool {
        if self.can_fire() {
            self.reload_timer.reset();
            true
        } else {
            false
        }
    }

    pub(crate) fn can_fire(&self) -> bool {
        self.reload_timer.finished()
    }
}

trait TowerUpdate {
    fn update(&mut self, time: Duration);
}

impl TowerUpdate for TowerComponents {
    fn update(&mut self, time: Duration) {
        self.reload_timer.tick(time);
    }
}
