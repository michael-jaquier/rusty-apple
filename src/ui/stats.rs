//! Stats

use bevy::sprite::Anchor;
use bevy_egui::{egui, EguiContexts};

use crate::{
    arena::grid::{GridResource, HighlightedSpot},
    mob::{enemy, EnemyComponent},
    prelude::*,
    towers::{TowerComponents, TowerInfo, TowerPosition},
    weapons::weapon::WeaponComponent,
};

/// The stats plugin.
pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system);
    }
}

// Use EGUI to draw the status of the tower and weapon on the screen
fn ui_system(
    mut context: EguiContexts,
    highlighted_spot: Res<HighlightedSpot>,
    tower_query: Query<(&TowerComponents, &WeaponComponent, &TowerPosition)>,
    grid: Res<GridResource>,
    tower_info: Res<TowerInfo>,
) {
    if let Some((x, y, z)) = highlighted_spot.0 {
        for (tower, weapon, pos) in tower_query.iter() {
            if z == pos.0 {
                let weapon_level = weapon.level;
                let mut projectile_data = weapon.projectile_data.clone();
                tower_info.mega_fire(&tower.tower, &mut projectile_data);
                let range = weapon
                    .weapon_type()
                    .range(&grid, tower_info.get_level(&tower.tower));
                let tower_damage = tower_info.get_damage(&tower.tower);
                egui::Window::new("Stats")
                    .collapsible(false)
                    .auto_sized()
                    .movable(false)
                    .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(0.0, 0.0))
                    .show(context.ctx_mut(), |ui| {
                        ui.label(format!("Weapon Level: {}", weapon_level));
                        ui.label(format!("{}", projectile_data));
                        ui.label(format!("Range: {}", range));
                        ui.label(format!("DPS: {}", tower_damage));
                    });
            }
        }
    }
}
