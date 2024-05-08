//! UI

use bevy::{
    input::mouse,
    prelude::*,
    render::view::window,
    utils::hashbrown::{HashMap, HashSet},
};
use bevy_egui::{
    egui::{self, debug_text::print, epaint::text::cursor},
    EguiContexts, EguiPlugin,
};
use enum_iterator::all;

use crate::{
    arena::{
        grid::{self, GridClickEvent, GridResource, HighlightedSpot},
        path_finding::Pos,
    },
    player,
    towers::{TowerData, TowerInfo, TowerTypes},
    weapons::weapon::{WeaponComponent, WeaponTypes},
};
use bevy_ecs_tilemap::prelude::*;

/// Input Plugin
pub use input::InputPlugin;
pub use stats::StatsPlugin;

pub(crate) mod input;
pub(crate) mod level;
pub(crate) mod stats;

/// Ui Plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_clicked)
            .add_systems(Update, ui_system)
            .add_systems(Update, track_mouse_position_system)
            .insert_resource(MousePosition::default())
            .add_plugins(level::LevelPlugin);
    }
}
#[derive(Debug, Default, Resource)]
pub(crate) struct MousePosition {
    position: Vec2,
}

fn track_mouse_position_system(
    mut mouse_position: ResMut<MousePosition>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    for event in cursor_moved_events.read() {
        mouse_position.position = event.position;
    }
}

fn get_screen_size(windows: Query<&Window>) -> Vec2 {
    let window = windows.single();
    Vec2::new(window.width(), window.height())
}

fn translate_mouse_coords(mouse_position: Vec2, screen_size: Vec2) -> Vec2 {
    Vec2::new(
        mouse_position.x - screen_size.x / 2.0,
        screen_size.y / 2.0 - mouse_position.y,
    )
}

fn map_to_grid(mouse_position: Vec2, grid: &GridResource) -> Option<(usize, usize)> {
    let mut closest_square = None;
    let mut min_distance = f32::MAX;

    let square_size = grid.grid_square_size; // assuming square_size is a field in GridResource

    for (x, row) in grid.grid_transform.iter().enumerate() {
        for (y, &coord) in row.iter().enumerate() {
            let distance = mouse_position.distance_squared(coord);
            if distance < min_distance {
                min_distance = distance;
                closest_square = Some((x, y));
            }
        }
    }

    // Check if the mouse position is within the grid size
    if let Some((x, y)) = closest_square {
        let half_grid_size = (grid.grid_size as f32 * square_size) / 2.0;
        let adjusted_mouse_x = mouse_position.x + half_grid_size;
        let adjusted_mouse_y = mouse_position.y + half_grid_size;

        if adjusted_mouse_x >= 0.0
            && adjusted_mouse_x <= (grid.grid_size as f32 * square_size)
            && adjusted_mouse_y >= 0.0
            && adjusted_mouse_y <= (grid.grid_size as f32 * square_size)
        {
            return Some((x, y));
        }
    }

    None
}
fn mouse_clicked(
    mut context: EguiContexts,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    grid: Res<GridResource>,
    window: Query<&Window>,
    mut grid_click_event: EventWriter<GridClickEvent>,
) {
    let screen_size = get_screen_size(window);
    for event in mouse_button_input.get_just_pressed() {
        if event == &mouse::MouseButton::Left {
            if context.ctx_mut().wants_pointer_input() {
                continue;
            }
            let translated_mouse_position =
                translate_mouse_coords(mouse_position.position, screen_size);

            // Check if the mouse click is within the grid space
            if let Some((x, y)) = map_to_grid(translated_mouse_position, &grid) {
                let pos = TilePos::new(x as u32, y as u32);
                let grid_coords = grid.grid_transform[x][y];
                let entity = grid.grid_entities[x][y];
                let transform = Transform::from_xyz(grid_coords.x, grid_coords.y, 0.0);

                grid_click_event.send(GridClickEvent::Highlight(entity, transform, pos));
            }
        }
    }
}

fn ui_system(
    mut context: EguiContexts,
    mut grid_click_event: EventWriter<GridClickEvent>,
    highlighted_spot: ResMut<HighlightedSpot>,
    tower_info: Res<TowerInfo>,
    player: Query<&player::Player>,
) {
    egui::Window::new("Weapons")
        .collapsible(false)
        .movable(false)
        .resizable(false)
        .auto_sized()
        .show(context.ctx_mut(), |ui| {
            let tower_types = all::<TowerTypes>().collect::<Vec<_>>();
            for tower in tower_types {
                let tower_cost = tower_info.get_cost(&tower);
                let player_bricks = player.single().bricks;
                let button_text = format!(
                    "Type: {:?} ( Level {} )",
                    tower,
                    tower_info.get_level(&tower)
                );
                let enabled = player_bricks >= tower_cost;
                let button = egui::Button::new(button_text);
                let button = if enabled {
                    button.stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                } else {
                    button.stroke(egui::Stroke::new(1.0, egui::Color32::DARK_RED))
                };

                if ui.add_enabled(enabled, button).clicked() {
                    if let Some((entity, transform, pos)) = highlighted_spot.0 {
                        grid_click_event.send(GridClickEvent::BuildTower(tower, transform, pos));
                        grid_click_event.send(GridClickEvent::DeHighlight(transform, pos));
                    }
                }
            }
        });
}

#[derive(Component)]
struct TileLabel;

#[derive(Component)]
struct HighlightedLabel;
