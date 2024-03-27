//! Grid

use bevy::{prelude::*, render::view::window};
use bevy_ecs_tilemap::{
    helpers::{filling::fill_tilemap, geometry::get_tilemap_center_transform},
    map::{
        TilemapGridSize, TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTileSize,
        TilemapType,
    },
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};
use bevy_egui::egui::debug_text::print;
use rand::Rng;

use crate::{
    assets::{BwTile, SpriteAssets, Tiles},
    towers::TowerTypes,
};

use super::{
    path_finding::{PathFindingEvent, Pos},
    ARENA_HEIGHT,
};
/// The grid plugin.
pub struct GridPlugin;

#[derive(Component)]
struct CurrentColor(u16);

#[derive(Debug, Resource, Default)]
pub(crate) struct GridResource {
    pub(crate) grid: Vec<Vec<bool>>,
    pub(crate) grid_size: u32,
    pub(crate) grid_square_size: f32,
    pub(crate) grid_coords: [(f32, f32); 4],
    pub(crate) grid_transform: Vec<Vec<Vec2>>,
    pub(crate) grid_enemy_start: TilePos,
    pub(crate) grid_enemy_end: TilePos,
    pub(crate) grid_entities: Vec<Vec<Entity>>,
    pub(crate) grid_towers: Vec<Vec<Entity>>,
}

impl GridResource {
    pub(crate) fn new(grid_size: u32, grid_square_size: f32, grid_coords: [(f32, f32); 4]) -> Self {
        let grid = vec![vec![false; grid_size as usize]; grid_size as usize];
        Self {
            grid,
            grid_size,
            grid_square_size,
            grid_coords,
            grid_transform: vec![vec![Vec2::ZERO; grid_size as usize]; grid_size as usize],
            grid_enemy_start: TilePos::new(10, 5),
            grid_enemy_end: TilePos::new(0, 5),
            grid_entities: vec![vec![]; grid_size as usize],
            grid_towers: vec![vec![]; grid_size as usize],
        }
    }

    pub(crate) fn set_grid(&mut self) {
        self.grid = vec![vec![false; self.grid_size as usize]; self.grid_size as usize];
    }

    pub(crate) fn set_occupied(&mut self, pos: &TilePos, entity: Entity) {
        self.grid[pos.x as usize][pos.y as usize] = true;
        self.grid_towers[pos.x as usize][pos.y as usize] = entity;
    }

    pub(crate) fn remove_occupied(&mut self, pos: &TilePos) {
        self.grid[pos.x as usize][pos.y as usize] = false;
        self.grid_towers[pos.x as usize][pos.y as usize] = Entity::PLACEHOLDER;
    }

    pub(crate) fn is_occupied(&self, pos: &TilePos) -> bool {
        self.grid[pos.x as usize][pos.y as usize]
    }

    pub(crate) fn get(&self, x: u32, y: u32) -> bool {
        self.grid[x as usize][y as usize]
    }

    pub(crate) fn get_tower(&self, pos: TilePos) -> Option<Entity> {
        let x = self.grid_towers[pos.x as usize][pos.y as usize];
        if x == Entity::PLACEHOLDER {
            None
        } else {
            Some(x)
        }
    }

    pub(crate) fn get_grid(&self) -> Vec<Vec<bool>> {
        self.grid.clone()
    }

    pub(crate) fn get_grid_size(&self) -> u32 {
        self.grid_size
    }

    pub(crate) fn get_grid_square_size(&self) -> f32 {
        self.grid_square_size
    }

    pub(crate) fn get_grid_coords(&self) -> [(f32, f32); 4] {
        self.grid_coords
    }

    pub(crate) fn top_left(&self) -> (f32, f32) {
        (self.grid_coords[0].0, self.grid_coords[0].1)
    }

    pub(crate) fn top_right(&self) -> (f32, f32) {
        (self.grid_coords[1].0, self.grid_coords[1].1)
    }

    pub(crate) fn bottom_left(&self) -> (f32, f32) {
        (self.grid_coords[2].0, self.grid_coords[2].1)
    }

    pub(crate) fn bottom_right(&self) -> (f32, f32) {
        (self.grid_coords[3].0, self.grid_coords[3].1)
    }

    pub(crate) fn set_top_left(&mut self, x: f32, y: f32) {
        self.grid_coords[0].0 = x;
        self.grid_coords[0].1 = y;
    }

    pub(crate) fn set_top_right(&mut self, x: f32, y: f32) {
        self.grid_coords[1].0 = x;
        self.grid_coords[1].1 = y;
    }

    pub(crate) fn set_bottom_left(&mut self, x: f32, y: f32) {
        self.grid_coords[2].0 = x;
        self.grid_coords[2].1 = y;
    }
    pub(crate) fn set_bottom_right(&mut self, x: f32, y: f32) {
        self.grid_coords[3].0 = x;
        self.grid_coords[3].1 = y;
    }

    pub(crate) fn width(&self) -> i32 {
        self.grid_size as i32
    }

    pub(crate) fn height(&self) -> i32 {
        self.grid_size as i32
    }

    pub(crate) fn successors(&self, pos: &TilePos) -> Vec<TilePos> {
        let mut successors = Vec::new();
        let width = self.width();
        let height = self.height();
        let dx = [0, 1, 0, -1];
        let dy = [1, 0, -1, 0];
        for i in 0..4 {
            let nx = pos.x as i32 + dx[i];
            let ny = pos.y as i32 + dy[i];
            if nx < 0 || nx >= height || ny < 0 || ny >= width {
                continue;
            }
            let nx = nx as u32;
            let ny = ny as u32;
            // Check if the cell is blocked
            if self.get(nx, ny) {
                continue;
            }
            successors.push(TilePos::new(nx, ny));
        }

        successors
    }
}
#[derive(Debug, Event)]
pub(crate) enum GridClickEvent {
    Highlight(Entity, Transform, TilePos),
    UpgradeTower(TowerTypes, TilePos),
    BuildTower(TowerTypes, Transform, TilePos),
    RemoveTower(TowerTypes, TilePos),
    DeHighlight(Transform, TilePos),
}

#[derive(Debug, Deref, Default, Resource)]
pub(crate) struct HighlightedSpot(pub(crate) Option<(Entity, Transform, TilePos)>);

#[derive(Debug, Default, Resource)]
pub(crate) struct HighlightedPaths(pub(crate) Vec<Entity>);

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HighlightedSpot::default())
            .insert_resource(GridResource::default())
            .insert_resource(HighlightedPaths::default())
            .add_event::<GridClickEvent>()
            .add_systems(Update, highight)
            .add_systems(Update, dehighlight)
            .add_systems(PostStartup, setup);
    }
}

fn setup(mut commands: Commands, mut grid: ResMut<GridResource>, bwtile: Res<Tiles>) {
    let square_size = 16.0; // Size of each square
    let padding = 150.0; // Padding around the grid
    let pad_height = ARENA_HEIGHT - padding;
    let squares = (pad_height / square_size).ceil() as i32;

    let line_length = square_size * squares as f32; // Length of the lines

    for y in 0..=squares {
        let line_transform =
            Transform::from_xyz(0.0, y as f32 * square_size - pad_height / 2.0, 0.0);

        if y == 0 {
            grid.set_bottom_left(
                line_transform.translation.x - line_length / 2.0,
                line_transform.translation.y,
            );

            grid.set_bottom_right(
                line_transform.translation.x + line_length / 2.0,
                line_transform.translation.y,
            );
        }
        if y == squares {
            grid.set_top_left(
                line_transform.translation.x - line_length / 2.0,
                line_transform.translation.y,
            );

            grid.set_top_right(
                line_transform.translation.x + line_length / 2.0,
                line_transform.translation.y,
            );
        }
    }

    let size = TilemapSize {
        x: squares as u32,
        y: squares as u32,
    };
    let tile_size = TilemapTileSize {
        x: square_size,
        y: square_size,
    };
    let grid_size = TilemapGridSize {
        x: square_size,
        y: square_size,
    };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(size);
    grid.grid_entities = vec![vec![Entity::PLACEHOLDER; size.y as usize]; size.x as usize];
    grid.grid_towers = vec![vec![Entity::PLACEHOLDER; size.y as usize]; size.x as usize];

    let mut grid_coords = vec![vec![Vec2::ZERO; size.y as usize]; size.x as usize];
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(4),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
            grid.grid_entities[x as usize][y as usize] = tile_entity;

            // calculate the center coordinates of the grid square
            let center_x = (x as f32 - size.x as f32 / 2.0 + 0.5) * square_size;
            let center_y = (y as f32 - size.y as f32 / 2.0 + 0.5) * square_size;
            grid_coords[x as usize][y as usize] = Vec2::new(center_x, center_y);
        }
    }
    grid.grid_transform = grid_coords;
    let map_type = TilemapType::default();
    let handle = bwtile.clone();

    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
            size,
            grid_size,
            map_type,
            tile_size,
            storage: tile_storage,
            spacing: TilemapSpacing { x: 0.0, y: 0.0 },
            texture: TilemapTexture::Single(handle),
            transform: get_tilemap_center_transform(&size, &grid_size, &map_type, -10.0),
            ..Default::default()
        })
        .insert(LastUpdate(0.0));

    grid.grid_size = squares as u32;
    grid.grid_square_size = square_size;
    grid.grid_enemy_end = TilePos::new(0, squares as u32 / 2);
    grid.grid_enemy_start = TilePos::new(squares as u32 - 1, squares as u32 / 2);
    grid.set_grid()
}

#[derive(Component)]
pub(crate) struct LastUpdate(f64);

fn highight(
    mut commands: Commands,
    mut grid_click_events: EventReader<GridClickEvent>,
    mut highlighted_spot: ResMut<HighlightedSpot>,
    mut highlighted_paths: ResMut<HighlightedPaths>,
    grid: Res<GridResource>,
    mut tile_query: Query<(Entity, &TilePos, &mut TileTextureIndex)>,
    mut path_finding_events: EventReader<PathFindingEvent>,
) {
    for event in path_finding_events.read() {
        match event {
            PathFindingEvent::HighlightCurrentPath(head, path) => {
                let grid_square_size = grid.get_grid_square_size();
                let x = head.x as f32 * grid_square_size + grid.bottom_left().0;
                let y = head.y as f32 * grid_square_size + grid.bottom_left().1;

                // Head Sprite
                let head_transform = Transform::from_xyz(x, y, 0.0);
                let offset = grid_square_size / 2.0;
                let head_offset_transform = Transform {
                    translation: Vec3::new(
                        head_transform.translation.x + offset,
                        head_transform.translation.y + offset,
                        head_transform.translation.z,
                    ),
                    ..head_transform
                };
                let entity = commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::MAROON,
                            custom_size: Some(Vec2::splat(grid_square_size / 4.0)),
                            ..Default::default()
                        },
                        transform: head_offset_transform,
                        ..Default::default()
                    })
                    .id();
                highlighted_paths.0.push(entity);

                // Path Sprites
                for pos in path {
                    tile_query
                        .get_mut(grid.grid_entities[pos.x as usize][pos.y as usize])
                        .unwrap()
                        .2
                         .0 = 5;
                }
            }
            _ => {}
        }
    }
    for event in grid_click_events.read() {
        match event {
            GridClickEvent::Highlight(entity, transform, pos) => {
                if let Some((entity, transform, tile_pos)) = highlighted_spot.0 {
                    if let Some((entity, pos, mut index)) = tile_query.get_mut(entity).ok() {
                        index.0 = 4;
                    }
                }

                if let Some((entity, tile_position, mut index)) = tile_query.get_mut(*entity).ok() {
                    index.0 = 2;
                    highlighted_spot.0 = Some((entity, transform.clone(), *tile_position));
                }
            }

            _ => {}
        }
    }
}

fn dehighlight(
    mut commands: Commands,
    mut grid_click_events: EventReader<GridClickEvent>,
    mut highlighted_spot: ResMut<HighlightedSpot>,
    mut tile_query: Query<(Entity, &TilePos, &mut TileTextureIndex)>,
) {
    for event in grid_click_events.read() {
        match event {
            GridClickEvent::DeHighlight(_transform, pos) => {
                for (entity, tile_position, mut index) in tile_query.iter_mut() {
                    if *pos == *tile_position {
                        index.0 = 4;
                    }
                }
                highlighted_spot.0 = None;
            }
            _ => {}
        }
    }
}
