//! Towers module.

use bevy::ecs::event;

use crate::{arena::grid::GridClickEvent, prelude::*};

/// Tower plugin.
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tower_system);
    }
}

fn tower_system(mut commands: Commands, mut grid_event: EventReader<GridClickEvent>) {
    for event in grid_event.read() {
        match event {
            GridClickEvent::Build(weapon, transform) => {
                println!("Build tower {:} at {:?}", weapon, transform);
            }
            _ => {}
        }
    }
}
