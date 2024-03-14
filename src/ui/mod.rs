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
    arena::grid::{self, GridClickEvent, HighlightedSpot, MouseClickEvent},
    towers::TowerTypes,
    weapon::{WeaponComponent, WeaponTypes, WeaponUpgradeEvent},
};

/// Ui Plugin
pub struct UiPlugin;

#[derive(Debug, Default, Resource)]
struct ShowWindow(bool);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_clicked)
            .add_systems(Update, ui_system)
            .insert_resource(ShowWindow::default());
    }
}

fn mouse_clicked(
    mut context: EguiContexts,
    mut show_window: ResMut<ShowWindow>,
    mut mouse_event: EventReader<MouseClickEvent>,
    mut grid_click_event: EventWriter<GridClickEvent>,
) {
    for event in mouse_event.read() {
        if event.button == mouse::MouseButton::Left {
            if context.ctx_mut().wants_pointer_input() {
                println!("Pointer input");
                continue;
            }
            show_window.0 = true;
            println!("Mouse clicked: {:?}", event.position.translation);
            grid_click_event.send(GridClickEvent::Highlight(event.position));
        }
    }
}

fn ui_system(
    mut context: EguiContexts,
    mut grid_click_event: EventWriter<GridClickEvent>,
    highlighted_spot: ResMut<HighlightedSpot>,
) {
    egui::Window::new("Weapons").show(context.ctx_mut(), |ui| {
        let tower_types = all::<TowerTypes>().collect::<Vec<_>>();
        for tower in tower_types {
            let button_text = format!("Tower {} ", tower);
            let button = egui::Button::new(button_text);
            if ui.add_enabled(true, button).clicked() {
                println!("Button clicked");
                if let Some((_, transform)) = highlighted_spot.0 {
                    println!("Tower built");
                    grid_click_event.send(GridClickEvent::Build(tower, transform));
                    grid_click_event.send(GridClickEvent::DeHighlight(transform));
                }
            }
        }
    });
}
