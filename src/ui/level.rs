//! Map level

use bevy_egui::{egui, EguiContexts};

use crate::prelude::*;

use super::ShowWindow;

#[derive(Debug, Clone, Resource)]
pub(crate) struct MapLevel {
    pub(crate) level: u32,
    timer: Timer,
}

impl Default for MapLevel {
    fn default() -> Self {
        MapLevel {
            level: 1,
            timer: Timer::from_seconds(45.0, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Clone, Event)]
pub(crate) struct LevelUpMap;

/// The level plugin.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapLevel::default())
            .add_event::<LevelUpMap>()
            .add_systems(Update, level_up_map_system)
            .add_systems(Update, ui_system);
    }
}

fn level_up_map_system(
    time: Res<Time>,
    mut level: ResMut<MapLevel>,
    mut level_up_map: EventReader<LevelUpMap>,
) {
    for _ in level_up_map.read() {
        level.level += 1;
        level.timer.reset();
    }

    if level.timer.tick(time.delta()).just_finished() {
        level.level += 1;
    }
}

// Use EGUI to draw the level and the level timer on the screen
fn ui_system(level: Res<MapLevel>, mut egui_contexts: EguiContexts) {
    egui::Window::new("Level")
        .collapsible(false)
        .show(egui_contexts.ctx_mut(), |ui| {
            ui.label(format!("Level: {}", level.level));
            ui.label(format!("Time: {:.2}", level.timer.elapsed().as_secs_f32()));
        });
}
