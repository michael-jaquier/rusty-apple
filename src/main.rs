#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::{Level, LogPlugin},
    prelude::*,
    window::WindowResolution,
};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPlugin;
use bevy_xpbd_2d::prelude::PhysicsPlugins;
use rusty_apple::{
    arena::{self, PathFindingPlugin, ARENA_HEIGHT, ARENA_WIDTH},
    assets, collision,
    mob::{self, MobPlugin},
    player, towers, ui,
    weapons::WeaponPlugin,
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
        .add_plugins(TilemapPlugin)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(arena::GridPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(MobPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(towers::TowerPlugin)
        .add_plugins(PathFindingPlugin)
        .add_plugins(WeaponPlugin)
        .add_plugins(collision::CollisionPlugin)
        .add_plugins(ui::InputPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(ui::StatsPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
