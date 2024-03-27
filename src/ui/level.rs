//! Map level

use bevy_egui::{egui, EguiContexts};

use crate::{
    mob::{enemy, EnemyComponent},
    prelude::*,
};

#[derive(Debug, Clone, Resource)]
pub(crate) struct MapLevel {
    pub(crate) level: u32,
}

impl Default for MapLevel {
    fn default() -> Self {
        MapLevel { level: 1 }
    }
}

#[derive(Debug, Clone, Event)]
pub(crate) enum LevelMap {
    LevelUp(u32),
    LevelDown(u32),
}

/// The level plugin.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapLevel::default())
            .add_event::<LevelMap>()
            .add_systems(Update, level_up_map_system)
            .add_systems(Update, ui_system);
    }
}

fn level_up_map_system(mut level: ResMut<MapLevel>, mut level_up_map: EventReader<LevelMap>) {
    for event in level_up_map.read() {
        match event {
            LevelMap::LevelUp(amount) => {
                level.level = level.level.saturating_add(*amount);
            }
            LevelMap::LevelDown(amount) => {
                level.level = level.level.saturating_sub(*amount);
            }
        }
    }
}

// Use EGUI to draw the level and the level timer on the screen
fn ui_system(
    level: Res<MapLevel>,
    mut egui_contexts: EguiContexts,
    enemy_component: Query<&EnemyComponent>,
    player: Query<&crate::player::Player>,
) {
    let kill_count = enemy_component
        .iter()
        .map(|enemy| enemy.spawner.current_kill)
        .sum::<usize>();
    let max_kill = enemy_component
        .iter()
        .map(|enemy| enemy.spawner.max_kill)
        .sum::<usize>();
    egui::Window::new("Game Stats")
        .collapsible(false)
        .show(egui_contexts.ctx_mut(), |ui| {
            ui.label(format!("Level: {}", level.level));
            ui.label(format!("Kills: {}/{}", kill_count, max_kill));
            ui.label(format!("Player HP: {}", player.single().hp));
            ui.label(format!("Player Bricks: {}", player.single().bricks));
        });
}
