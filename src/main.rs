#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use bevy::{prelude::*, window::WindowResolution};
use bevy_xpbd_2d::prelude::PhysicsPlugins;
use rusty_apple::{assets, player, weapon};

const ARENA_WIDTH: f32 = 1280.0;
const ARENA_HEIGHT: f32 = 800.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kataster".to_string(),
                resolution: WindowResolution::new(ARENA_WIDTH, ARENA_HEIGHT),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(weapon::WeaponPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
