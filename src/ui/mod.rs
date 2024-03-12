//! UI

use bevy::{
    prelude::*,
    utils::hashbrown::{HashMap, HashSet},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use enum_iterator::all;

use crate::{
    player::PlayerComponent,
    weapon::{WeaponComponents, WeaponTypes, WeaponUpgradeEvent},
};

/// Ui Plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_example_system);
    }
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut player_query: Query<(Entity, &PlayerComponent, &mut WeaponComponents)>,
    mut weapon_upgrade_events: EventWriter<WeaponUpgradeEvent>,
) {
    egui::Window::new("Weapons").show(contexts.ctx_mut(), |ui| {
        let mut player_points = 0;
        let mut weapon_points = HashMap::new();
        for (_, pc, wc) in player_query.iter() {
            ui.label(format!("Points: {}", pc.points));
            player_points = pc.points;
            wc.weapons.iter().for_each(|weapon| {
                weapon_points.insert(weapon.projectile_data.weapon_type, weapon.cost);
            });
        }

        let weapon_types = all::<WeaponTypes>().collect::<Vec<_>>();
        for weapon_type in weapon_types {
            let button_text = format!(
                "Weapon {} :: Cost: {}",
                weapon_type,
                weapon_points.get(&weapon_type).unwrap_or(&0)
            );
            let button = egui::Button::new(button_text);
            if player_points >= *weapon_points.get(&weapon_type).unwrap_or(&0) {
                if ui.add_enabled(true, button).clicked() {
                    for (pe, _pc, _wc) in player_query.iter_mut() {
                        weapon_upgrade_events.send(WeaponUpgradeEvent {
                            weapon: weapon_type,
                            player: pe,
                        });
                    }
                };
            } else {
                ui.add_enabled(false, button);
            }
        }
    });
}
