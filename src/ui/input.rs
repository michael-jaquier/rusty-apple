//! Users input keys

use bevy::{
    input::keyboard,
    prelude::*,
    utils::{dbg, info},
};
use leafwing_input_manager::{
    action_state::{self, ActionState},
    clashing_inputs::ClashStrategy,
    input_map::{self, InputMap},
    plugin::InputManagerPlugin,
    Actionlike, InputManagerBundle,
};
use pathfinding::grid;

use crate::{
    arena::grid::{GridClickEvent, HighlightedSpot},
    towers::{self, TowerComponents, TowerPosition, TowerTypes},
};

/// Plugin to handle user input
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_input_manager)
            .add_systems(Update, keyboard_input_system)
            .add_systems(Update, keyboard_action_system)
            .add_plugins(InputManagerPlugin::<TowerTypes>::default())
            .add_plugins(InputManagerPlugin::<ActionKeys>::default())
            .insert_resource(ClashStrategy::PrioritizeLongest);
    }
}

#[derive(Debug, Default, Component)]
struct BuildMode(bool);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Actionlike, Reflect)]
enum ActionKeys {
    RemoveTower,
    UpgradeTower,
}

fn spawn_input_manager(mut commands: Commands) {
    use towers::TowerTypes::*;
    use KeyCode::*;

    let mut input_map = InputMap::default();
    let mut action_map = InputMap::default();

    input_map.insert(Basic, KeyB);
    input_map.insert(Fire, KeyF);
    input_map.insert(Ice, KeyI);
    input_map.insert(Sniper, KeyS);
    action_map.insert(ActionKeys::RemoveTower, Backspace);
    action_map.insert(ActionKeys::UpgradeTower, KeyU);

    commands.spawn(InputManagerBundle::with_map(input_map));
    commands.spawn(InputManagerBundle::with_map(action_map));
}

fn keyboard_input_system(
    keyboard_input: Query<&ActionState<TowerTypes>>,
    highlightedspot: Res<HighlightedSpot>,
    mut grid_click_events: EventWriter<GridClickEvent>,
) {
    let action = keyboard_input.single();
    for act in action.get_just_pressed() {
        if let Some((_, transform, pos)) = highlightedspot.0 {
            grid_click_events.send(GridClickEvent::BuildTower(act.clone(), transform, pos));
        }
    }
}

fn keyboard_action_system(
    keyboard_input: Query<&ActionState<ActionKeys>>,
    highlightedspot: Res<HighlightedSpot>,
    mut grid_click_events: EventWriter<GridClickEvent>,
    tower_query: Query<(&TowerComponents, &TowerPosition)>,
) {
    let action = keyboard_input.single();
    for act in action.get_just_pressed() {
        match act {
            ActionKeys::RemoveTower => {
                if let Some((_, _, pos)) = highlightedspot.0 {
                    for (tower, tower_pos) in tower_query.iter() {
                        if tower_pos.0 == pos {
                            grid_click_events.send(GridClickEvent::RemoveTower(tower.tower, pos));
                        }
                    }
                }
            }
            ActionKeys::UpgradeTower => {
                if let Some((_, _, pos)) = highlightedspot.0 {
                    for (tower, tower_pos) in tower_query.iter() {
                        if tower_pos.0 == pos {
                            grid_click_events.send(GridClickEvent::UpgradeTower(tower.tower, pos));
                        }
                    }
                }
            }
        }
    }
}
