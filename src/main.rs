#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::EguiPlugin;
use bevy_xpbd_2d::prelude::PhysicsPlugins;
use rusty_apple::{
    arena::{self, PathFindingPlugin, ARENA_HEIGHT, ARENA_WIDTH},
    assets, mob, towers, ui, weapon,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty-Apple".to_string(),
                resolution: WindowResolution::new(ARENA_WIDTH, ARENA_HEIGHT),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(arena::GridPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(mob::MobPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(towers::TowerPlugin)
        .add_plugins(PathFindingPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
