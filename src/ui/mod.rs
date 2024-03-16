//! UI

use bevy::{
    input::mouse,
    prelude::*,
    utils::hashbrown::{HashMap, HashSet},
};
use bevy_egui::{
    egui::{self, debug_text::print},
    EguiContexts, EguiPlugin,
};
use enum_iterator::all;

use crate::{
    arena::{
        grid::{self, GridClickEvent, GridResource, HighlightedSpot},
        path_finding::Pos,
    },
    towers::TowerTypes,
    weapons::weapon::{WeaponComponent, WeaponTypes},
};

pub(crate) mod level;

/// Ui Plugin
pub struct UiPlugin;

#[derive(Debug, Default, Resource)]
struct ShowWindow(bool);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_clicked)
            .add_systems(Update, ui_system)
            .add_systems(Update, track_mouse_position_system)
            .insert_resource(ShowWindow::default())
            .insert_resource(MousePosition::default())
            .add_event::<MouseClickEvent>()
            .add_plugins(level::LevelPlugin);
    }
}
#[derive(Debug, Default, Resource)]
pub(crate) struct MousePosition {
    position: Vec2,
}

#[derive(Debug, Event)]
pub(crate) struct MouseClickEvent {
    pub(crate) position: Transform,
    pub(crate) grid_position: Pos,
    pub(crate) button: MouseButton,
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

fn normalize_to_grid(mouse_position: Vec2, grid: &GridResource) -> (i32, i32) {
    let x = (mouse_position.x - grid.bottom_left().0) / grid.grid_square_size;
    let y = (mouse_position.y - grid.bottom_left().1) / grid.grid_square_size;
    (x as i32, y as i32)
}

fn mouse_clicked(
    mut context: EguiContexts,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    grid: Res<GridResource>,
    window: Query<&Window>,
    mut mouse_writer: EventWriter<MouseClickEvent>,
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
            let grid_coords = normalize_to_grid(translated_mouse_position, &grid);
            let pos = Pos::new(grid_coords.0 as usize, grid_coords.1 as usize);

            // Check if the mouse click is within the grid space
            if grid_coords.0 < grid.width()
                && grid_coords.1 < grid.height()
                && grid_coords.0 >= 0
                && grid_coords.1 >= 0
            {
                let transform = Transform::from_xyz(
                    grid.bottom_left().0 + grid_coords.0 as f32 * grid.grid_square_size,
                    grid.bottom_left().1 + grid_coords.1 as f32 * grid.grid_square_size,
                    0.0,
                );

                mouse_writer.send(MouseClickEvent {
                    position: transform,
                    grid_position: pos,
                    button: *event,
                });

                grid_click_event.send(GridClickEvent::Highlight(transform, pos));
            }
        }
    }
}

fn ui_system(
    mut context: EguiContexts,
    mut grid_click_event: EventWriter<GridClickEvent>,
    highlighted_spot: ResMut<HighlightedSpot>,
) {
    egui::Window::new("Weapons")
    .collapsible(false)
    .movable(false)
    
    
    .show(context.ctx_mut(), |ui| {
        let tower_types = all::<TowerTypes>().collect::<Vec<_>>();
        for tower in tower_types {
            let button_text = format!("Tower {} ", tower);
            let button = egui::Button::new(button_text);
            if ui.add_enabled(true, button).clicked() {
                if let Some((_entity, transform, pos)) = highlighted_spot.0 {
                    grid_click_event.send(GridClickEvent::Build(tower, transform, pos));
                    grid_click_event.send(GridClickEvent::DeHighlight(transform, pos));
                }
            }
        }
    });
}
