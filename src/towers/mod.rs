//! Towers module.

use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_egui::egui::Pos2;
use bevy_egui::egui::Shape;
use bevy_egui::egui::Stroke;
use enum_iterator::all;
use enum_iterator::Sequence;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;

use crate::arena::grid::GridResource;
use crate::arena::grid::HighlightedSpot;
use crate::arena::path_finding;
use crate::arena::GRID_SQUARE_SIZE;
use crate::collision::GameLayer;
use crate::mob::EnemyUnit;
use crate::player::Player;
use crate::player::PlayerUpdateEvent;
use crate::towers::path_finding::path_finding;
use crate::weapons::weapon::ProjectileData;
use crate::weapons::weapon::WeaponComponent;
use crate::weapons::weapon::WeaponTypes;
use crate::weapons::weapon::WeaponUpdate;
use crate::weapons::FireWeaponEvent;
use crate::{arena::grid::GridClickEvent, assets::SpriteAssets, prelude::*};
#[derive(Debug, Copy, Clone)]
pub(crate) enum TowerLevelUpReason {
    Kill,
    Upgrade,
}

#[derive(Debug, Component, Clone, PartialEq, Eq, Default, Copy)]
pub(crate) struct TowerStatusEffects {
    pub(crate) potency: u32,
    pub(crate) duration: u32,
}

#[derive(Debug, Component, Clone, PartialEq)]
pub(crate) struct TowerData {
    pub(crate) experience: u32,
    level: u32,
    status_effect: TowerStatusEffects,
    current_cost: u32,
    tower_count: u32,
    tower_type: TowerTypes,
}

impl TowerData {
    pub(crate) fn new(tower_type: &TowerTypes) -> Self {
        Self {
            experience: 0,
            level: 1,
            status_effect: TowerStatusEffects::default(),
            current_cost: 1,
            tower_count: 0,
            tower_type: *tower_type,
        }
    }

    pub(crate) fn get_level(&self) -> u32 {
        self.level
    }
    pub(crate) fn add_experience(&mut self, experience: u32) {
        self.experience += experience;
        while self.experience > self.level * 333 {
            self.experience -= self.level;
            self.level += 1;
            self.mutate_status()
        }
    }

    fn mutate_status(&mut self) {
        self.status_effect.duration = self.status_effect.duration.saturating_add(1);
        self.status_effect.potency = self.status_effect.potency.saturating_add(1);
    }

    pub(crate) fn get_status(&self) -> &TowerStatusEffects {
        &self.status_effect
    }

    pub(crate) fn build_tower(&mut self) {
        self.tower_count += 1;
        self.current_cost = self.tower_type.cost(self.tower_count);
    }

    pub(crate) fn remove_tower(&mut self) {
        self.tower_count -= 1;
        self.current_cost = self.tower_type.cost(self.tower_count);
    }

    pub(crate) fn upgrade(&mut self) {
        // Get experience needed for next level
        let experience_needed = self.level * 333;
        self.add_experience(experience_needed);
    }
}

#[derive(Debug, Resource)]
pub(crate) struct TowerInfo {
    pub(crate) tower_data: HashMap<TowerTypes, TowerData>,
}

impl TowerInfo {
    fn next_level(&self, tower: &TowerTypes) -> u32 {
        self.get_level(tower) + 1
    }

    pub(crate) fn get_level(&self, tower: &TowerTypes) -> u32 {
        if let Some(data) = self.tower_data.get(tower) {
            return data.get_level();
        }
        1
    }

    pub(crate) fn add_experience(&mut self, tower: &TowerTypes, experience: u32) -> u32 {
        let previous_level = self.get_level(tower);
        if let Some(data) = self.tower_data.get_mut(tower) {
            data.add_experience(experience)
        }
        let current = self.get_level(tower);
        if current < previous_level {
            error!("Tower level decreased");
        }
        current.saturating_sub(previous_level)
    }

    pub(crate) fn upgrade(&mut self, tower: &TowerTypes) {
        if let Some(data) = self.tower_data.get_mut(tower) {
            data.level += 1;
            data.mutate_status();
        }
    }

    pub(crate) fn get_data(&self, tower: &TowerTypes) -> TowerData {
        self.tower_data
            .get(tower)
            .cloned()
            .expect("Tower not found")
    }

    pub(crate) fn build_tower(&mut self, tower: &TowerTypes) {
        if let Some(data) = self.tower_data.get_mut(tower) {
            data.build_tower()
        }
    }

    pub(crate) fn remove_tower(&mut self, tower: &TowerTypes) {
        if let Some(data) = self.tower_data.get_mut(tower) {
            data.remove_tower()
        }
    }
    pub(crate) fn get_cost(&self, tower: &TowerTypes) -> u32 {
        self.get_data(tower).current_cost
    }

    pub(crate) fn enough_bricks(&self, tower: &TowerTypes, player: &Player) -> Option<u32> {
        if player.bricks < self.get_cost(tower) {
            None
        } else {
            Some(self.get_cost(tower))
        }
    }

    pub(crate) fn mega_fire(&self, tower: &TowerTypes, projectile_data: &mut ProjectileData) {
        let level = self.get_level(tower);
        projectile_data.damage += level as usize;
        projectile_data.count += (level / 11) as usize;
    }
}

impl Default for TowerInfo {
    fn default() -> Self {
        let towers = all::<TowerTypes>();
        let tower_data = towers
            .map(|tower| (tower, TowerData::new(&tower)))
            .collect();

        Self { tower_data }
    }
}

#[derive(Debug, Event, Clone, Copy)]
pub(crate) struct TowerLevelUp {
    pub(crate) entity: Entity,
    pub(crate) reason: TowerLevelUpReason,
    pub(crate) enemy_experience: usize,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct TowerGizmos {}
/// Tower plugin.
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_tower_range_system);
        app.add_systems(Update, tower_system)
            .add_event::<TowerLevelUp>()
            .init_gizmo_group::<TowerGizmos>()
            .insert_resource(TowerInfo::default())
            .add_systems(Update, tower_fire_system)
            .add_systems(Update, tower_level_up_system)
            .add_systems(Update, tower_upgrade_system);
    }
}

#[derive(Debug, Component, Deref, DerefMut, Eq, PartialEq)]
pub(crate) struct TowerPosition(pub(crate) TilePos);

fn tower_upgrade_system(
    mut grid_event: EventReader<GridClickEvent>,
    mut tower_level_up_event: EventWriter<TowerLevelUp>,
    mut player_event: EventWriter<PlayerUpdateEvent>,
    tower_info: ResMut<TowerInfo>,
    player: Query<&Player>,
) {
    for event in grid_event.read() {
        match event {
            GridClickEvent::UpgradeTower(tower_type) => {
                if let Some(bricks) = tower_info.enough_bricks(tower_type, &player.single()) {
                    tower_level_up_event.send(TowerLevelUp {
                        entity: Entity::PLACEHOLDER,
                        reason: TowerLevelUpReason::Upgrade,
                        enemy_experience: 0,
                    });
                    player_event.send(PlayerUpdateEvent::Build(bricks));
                }
            }

            _ => {}
        }
    }
}

fn tower_system(
    mut commands: Commands,
    mut grid_event: EventReader<GridClickEvent>,
    mut player_event: EventWriter<PlayerUpdateEvent>,
    assets: Res<SpriteAssets>,
    mut grid: ResMut<GridResource>,
    mut tower_info: ResMut<TowerInfo>,
    player: Query<&Player>,
) {
    for event in grid_event.read() {
        match event {
            GridClickEvent::BuildTower(tower_type, transform, pos) => {
                let bricks = tower_info.get_cost(tower_type);
                let player_bricks = player.single().bricks;
                if bricks > player_bricks {
                    trace!("Not enough bricks");
                    continue;
                }
                if grid.is_occupied(pos) {
                    trace!("Position is occupied");
                    continue;
                }

                // Ensure tower position wont cause no path
                grid.set_occupied(pos, Entity::PLACEHOLDER);
                if path_finding(&grid, grid.grid_enemy_start).is_none() {
                    trace!("Tower position will cause no path");
                    grid.remove_occupied(pos);
                    continue;
                };
                let image = assets.tower_sprites[tower_type].clone();
                let tower_component = TowerComponents { tower: *tower_type };

                let weapon_component: WeaponComponent = WeaponComponent::from(*tower_type);

                let entity = commands
                    .spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(GRID_SQUARE_SIZE)),
                                ..Default::default()
                            },
                            transform: transform.with_scale(Vec3::splat(0.5)),
                            texture: image.clone(),

                            ..Default::default()
                        },
                        TowerPosition(*pos),
                        tower_component,
                        weapon_component,
                        CollisionLayers::new(GameLayer::Tower, [GameLayer::Enemy]),
                    ))
                    .id();

                grid.set_occupied(pos, entity);
                tower_info.build_tower(tower_type);
                player_event.send(PlayerUpdateEvent::Build(bricks));
            }
            GridClickEvent::RemoveTower(tower_type, tile_pos) => {
                if let Some(entity) = grid.get_tower(*tile_pos) {
                    commands.entity(entity).despawn();
                    grid.remove_occupied(tile_pos);
                    tower_info.remove_tower(tower_type);
                }
            }

            _ => {}
        }
    }
}

fn draw_tower_range_system(
    tower_query: Query<(
        Entity,
        &TowerComponents,
        &WeaponComponent,
        &Transform,
        &TowerPosition,
    )>,
    tower_info: Res<TowerInfo>,
    grid: Res<GridResource>,
    mut my_gizmos: Gizmos<TowerGizmos>,
    highlighted_spot: Res<HighlightedSpot>,
) {
    if let Some((_, transform, tile_pos)) = highlighted_spot.0 {
        for (entity, tower, weapon, transform, tower_position) in tower_query.iter() {
            if tower_position.0 != tile_pos {
                continue;
            }
            let tower_position = transform.translation;
            let range = weapon
                .weapon_type()
                .range(&grid, tower_info.get_level(&tower.tower));

            my_gizmos
                .circle_2d(tower_position.truncate(), range, Color::NAVY)
                .segments(64);
        }
    }
}

fn tower_fire_system(
    time: Res<Time>,
    mut tower_query: Query<(
        Entity,
        &mut TowerComponents,
        &mut WeaponComponent,
        &Transform,
    )>,
    enemies_position: Query<(Entity, &Transform), With<EnemyUnit>>,
    mut fire_event_writer: EventWriter<FireWeaponEvent>,
    tower_info: Res<TowerInfo>,
    grid: Res<GridResource>,
) {
    for (entity, tower, mut weapon, transform) in tower_query.iter_mut() {
        weapon.update(time.delta());
        if let Some(mut projectile_data) = weapon.fire() {
            // Check how many projectiles to fire
            let mut enemies_to_target = Vec::new();
            let mut enemies_targeted = HashSet::new();
            for _ in 0..projectile_data.count {
                let tower_position = transform.translation;
                let mut nearest_distance = f32::MAX;
                let futhest_disance = weapon
                    .weapon_type()
                    .range(&grid, tower_info.get_level(&tower.tower));

                for (entity, enemy_transform) in enemies_position.iter() {
                    if enemies_targeted.contains(&entity) {
                        continue;
                    }
                    let enemy_position = enemy_transform.translation;
                    let distance = tower_position.distance(enemy_position);

                    if distance < nearest_distance && distance < futhest_disance {
                        nearest_distance = distance;
                        enemies_to_target.push(enemy_position);
                        enemies_targeted.insert(entity);
                    }
                }
            }
            // Find the nearest enemy
            let tower_position = transform.translation;

            for nearest_enemy_position in enemies_to_target {
                let direction = (nearest_enemy_position - tower_position).normalize();
                let velocity = (direction * projectile_data.speed_multiplier).truncate(); // Set the speed as needed
                projectile_data.source_entity = Some(entity);
                tower_info.mega_fire(&tower.tower, &mut projectile_data);

                fire_event_writer.send(FireWeaponEvent {
                    weapon_projectile_data: projectile_data,
                    source_transform: *transform,
                    velocity: LinearVelocity(velocity),
                    source_entity: entity,
                });
            }
        }
    }
}

fn tower_level_up_system(
    mut tower_level_up_event: EventReader<TowerLevelUp>,
    mut tower_query: Query<(Entity, &mut TowerComponents, &mut WeaponComponent)>,
    mut tower_datum: ResMut<TowerInfo>,
) {
    let mut tower_levels = None;
    for event in tower_level_up_event.read() {
        match event {
            TowerLevelUp {
                entity,
                reason: TowerLevelUpReason::Kill,
                enemy_experience,
            } => {
                let mut tower = None;
                if let Some((entity, tc, wc)) = tower_query.get(event.entity).ok() {
                    tower = Some(tc.tower);
                }

                if let Some(tower) = tower {
                    let level = tower_datum.add_experience(&tower, *enemy_experience as u32);
                    tower_levels = Some((tower, level));
                }
            }
            TowerLevelUp {
                entity,
                reason: TowerLevelUpReason::Upgrade,
                enemy_experience,
            } => {
                if let Some((entity, tc, wc)) = tower_query.get(event.entity).ok() {
                    tower_datum.upgrade(&tc.tower);
                    let tower_type = tc.tower;
                    tower_levels = Some((tower_type, 1));
                }
            }
        }
    }

    if let Some((tower, times)) = tower_levels {
        for (entity, mut tc, mut wc) in tower_query.iter_mut() {
            for _ in 0..times {
                if tc.tower == tower {
                    wc.level_up();
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Sequence, Copy, Actionlike, Reflect)]
pub(crate) enum TowerTypes {
    Basic,
    Fire,
    Ice,
    Sniper,
}
impl Display for TowerTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TowerTypes {
    pub(crate) fn set(sprite_assets: &mut SpriteAssets, asset_server: Res<AssetServer>) {
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Basic, asset_server.load("basic_tower.png"));
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Fire, asset_server.load("fire_tower.png"));
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Ice, asset_server.load("ice_tower.png"));
        sprite_assets
            .tower_sprites
            .insert(TowerTypes::Sniper, asset_server.load("sniper_tower.png"));
    }

    pub(crate) fn cost(&self, scale: u32) -> u32 {
        match self {
            TowerTypes::Basic => 1 + scale,
            TowerTypes::Fire => 2 + scale * 5,
            TowerTypes::Ice => 3 * scale,
            TowerTypes::Sniper => 15 * scale,
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub(crate) struct TowerComponents {
    pub(crate) tower: TowerTypes,
}

impl From<TowerTypes> for WeaponTypes {
    fn from(value: TowerTypes) -> Self {
        match value {
            TowerTypes::Basic => WeaponTypes::Laser,
            TowerTypes::Fire => WeaponTypes::Fire,
            TowerTypes::Ice => WeaponTypes::Ice,
            TowerTypes::Sniper => WeaponTypes::Rifle,
        }
    }
}

impl From<TowerTypes> for ProjectileData {
    fn from(value: TowerTypes) -> Self {
        let weapon: WeaponTypes = value.into();
        weapon.into()
    }
}

impl From<TowerTypes> for WeaponComponent {
    fn from(value: TowerTypes) -> Self {
        let weapon_type: WeaponTypes = value.into();
        weapon_type.into()
    }
}
