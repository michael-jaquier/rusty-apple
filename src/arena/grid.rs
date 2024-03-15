//! Grid

use bevy::{
    input::mouse::{self, MouseButtonInput},
    prelude::*,
    reflect::OffsetAccess,
    render::view::window,
    sprite::Anchor,
};
use bevy_egui::egui::debug_text::print;
use bevy_xpbd_2d::parry::na::ComplexField;
use leafwing_input_manager::user_input::InputKind;

use crate::{towers::TowerTypes, weapon::WeaponTypes};

use super::{
    path_finding::{PathFindingEvent, Pos},
    ARENA_HEIGHT, ARENA_WIDTH, GRID_SQUARE_SIZE,
};
/// The grid plugin.
pub struct GridPlugin;

#[derive(Debug, Resource, Default)]
pub(crate) struct GridResource {
    pub(crate) grid: Vec<Vec<bool>>,
    pub(crate) grid_size: usize,
    pub(crate) grid_square_size: f32,
    pub(crate) grid_coords: [(f32, f32); 4],
    pub(crate) grid_enemy_start: Pos,
    pub(crate) grid_enemy_end: Pos,
}

impl GridResource {
    pub(crate) fn new(
        grid_size: usize,
        grid_square_size: f32,
        grid_coords: [(f32, f32); 4],
    ) -> Self {
        let grid = vec![vec![false; grid_size]; grid_size];
        Self {
            grid,
            grid_size,
            grid_square_size,
            grid_coords,
            grid_enemy_start: Pos::new(10, 5),
            grid_enemy_end: Pos::new(0, 5),
        }
    }

    pub(crate) fn set_grid(&mut self) {
        self.grid = vec![vec![false; self.grid_size]; self.grid_size];
    }

    pub(crate) fn set_occupied(&mut self, pos: &Pos, value: bool) {
        self.grid[pos.x()][pos.y()] = value;
    }

    pub(crate) fn get(&self, x: usize, y: usize) -> bool {
        self.grid[x][y]
    }

    pub(crate) fn get_grid(&self) -> Vec<Vec<bool>> {
        self.grid.clone()
    }

    pub(crate) fn get_grid_size(&self) -> usize {
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

    pub(crate) fn successors(&self, pos: &Pos) -> Vec<Pos> {
        let mut successors = Vec::new();
        let width = self.width();
        let height = self.height();
        let dx = [0, 1, 0, -1];
        let dy = [1, 0, -1, 0];
        for i in 0..4 {
            let nx = pos.x() as i32 + dx[i];
            let ny = pos.y() as i32 + dy[i];
            if nx < 0 || nx >= height || ny < 0 || ny >= width {
                continue;
            }
            // Check if the cell is blocked
            if self.get(nx as usize, ny as usize) {
                continue;
            }
            successors.push(Pos::new(nx as usize, ny as usize));
        }

        successors
    }
}
#[derive(Debug, Event)]
pub(crate) enum GridClickEvent {
    Highlight(Transform, Pos),
    Upgrade(TowerTypes, Transform, Pos),
    Build(TowerTypes, Transform, Pos),
    DeHighlight(Transform, Pos),
}

#[derive(Debug, Default, Resource)]
pub(crate) struct HighlightedSpot(pub(crate) Option<(Entity, Transform, Pos)>);

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
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, mut grid: ResMut<GridResource>) {
    let square_size = 50.0; // Size of each square
    let padding = 250.0; // Padding around the grid
    let pad_height = ARENA_HEIGHT - padding;
    let squares = (pad_height / square_size).ceil() as i32;

    let line_thickness = 1.0; // Thickness of the lines
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

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(Vec2::new(line_length, line_thickness)),
                ..Default::default()
            },
            transform: line_transform,
            ..Default::default()
        });

        let line_transform =
            Transform::from_xyz(y as f32 * square_size - pad_height / 2.0, 0.0, 0.0);

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(line_thickness, line_length)),
                ..Default::default()
            },
            transform: line_transform,
            ..Default::default()
        });
    }

    grid.grid_size = squares as usize;
    grid.grid_square_size = square_size;
    grid.grid_enemy_end = Pos::new(0, squares as usize / 2);
    grid.grid_enemy_start = Pos::new(squares as usize - 1, squares as usize / 2);
    grid.set_grid()
}

fn highight(
    mut commands: Commands,
    mut grid_click_events: EventReader<GridClickEvent>,
    mut highlighted_spot: ResMut<HighlightedSpot>,
    mut highlighted_paths: ResMut<HighlightedPaths>,
    mut path_finding_events: EventReader<PathFindingEvent>,
    grid: Res<GridResource>,
) {
    for event in path_finding_events.read() {
        match event {
            PathFindingEvent::HighlightCurrentPath(head, path) => {
                {
                    for entity in highlighted_paths.0.drain(..) {
                        let maybe_entity = commands.get_entity(entity);
                        if let Some(mut entity) = maybe_entity {
                            entity.despawn();
                        }
                    }
                }
                let grid_square_size = grid.get_grid_square_size();
                let x = head.x() as f32 * grid_square_size + grid.bottom_left().0;
                let y = head.y() as f32 * grid_square_size + grid.bottom_left().1;

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
                    let x = pos.x() as f32 * grid_square_size + grid.bottom_left().0;
                    let y = pos.y() as f32 * grid_square_size + grid.bottom_left().1;

                    let offset = grid_square_size / 2.0;
                    let transform = Transform::from_xyz(x, y, 0.0);
                    let offset_transform = Transform {
                        translation: Vec3::new(
                            transform.translation.x + offset,
                            transform.translation.y + offset,
                            transform.translation.z,
                        ),
                        ..transform
                    };

                    let entity = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::SILVER,
                                custom_size: Some(Vec2::splat(grid_square_size / 4.0)),
                                ..Default::default()
                            },
                            transform: offset_transform,
                            ..Default::default()
                        })
                        .id();
                    highlighted_paths.0.push(entity);
                }
            }
            _ => {}
        }
    }

    for event in grid_click_events.read() {
        match event {
            GridClickEvent::Highlight(transform, pos) => {
                if let Some(entity) = highlighted_spot.0 {
                    let maybe_entity = commands.get_entity(entity.0);
                    if let Some(mut entity) = maybe_entity {
                        entity.despawn();
                    }
                }

                let grid_square_size = grid.get_grid_square_size();
                let offset_transform = Transform {
                    translation: Vec3::new(
                        transform.translation.x + grid_square_size / 2.0,
                        transform.translation.y + grid_square_size / 2.0,
                        transform.translation.z,
                    ),
                    ..*transform
                };
                let entity = commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            custom_size: Some(Vec2::splat(grid_square_size)),
                            ..Default::default()
                        },
                        transform: offset_transform,
                        ..Default::default()
                    })
                    .id();
                highlighted_spot.0 = Some((entity, offset_transform, *pos));
            }

            _ => {}
        }
    }
}

fn dehighlight(
    mut commands: Commands,
    mut grid_click_events: EventReader<GridClickEvent>,
    mut highlighted_spot: ResMut<HighlightedSpot>,
) {
    for event in grid_click_events.read() {
        match event {
            GridClickEvent::DeHighlight(_transform, pos) => {
                if let Some((entity, _, _)) = highlighted_spot.0 {
                    let maybe_entity = commands.get_entity(entity);
                    if let Some(mut entity) = maybe_entity {
                        entity.despawn();
                    }
                }
                highlighted_spot.0 = None;
            }
            _ => {}
        }
    }
}
