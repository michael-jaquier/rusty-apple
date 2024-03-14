//! Grid

use bevy::{
    input::mouse::{self, MouseButtonInput},
    prelude::*,
    render::view::window,
    sprite::Anchor,
};
use bevy_egui::egui::debug_text::print;
use bevy_xpbd_2d::parry::na::ComplexField;
use leafwing_input_manager::user_input::InputKind;

use crate::{towers::TowerTypes, weapon::WeaponTypes};

use super::{path_finding::Pos, ARENA_HEIGHT, ARENA_WIDTH, GRID_SQUARE_SIZE};
/// The grid plugin.
pub struct GridPlugin;

#[derive(Debug, Default, Resource)]
struct MousePosition {
    position: Vec2,
}

#[derive(Debug, Event)]
pub(crate) struct MouseClickEvent {
    pub(crate) position: Transform,
    pub(crate) button: MouseButton,
}

#[derive(Debug, Resource, Default)]
pub(crate) struct GridResource {
    pub(crate) grid: Vec<Vec<bool>>,
    pub(crate) grid_size: usize,
    pub(crate) grid_square_size: f32,
    pub(crate) grid_coords: [(f32, f32); 4],
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
        }
    }

    pub(crate) fn set_grid(&mut self) {
        self.grid = vec![vec![false; self.grid_size]; self.grid_size];
    }

    pub(crate) fn set(&mut self, x: usize, y: usize, value: bool) {
        self.grid[x][y] = value;
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
    Highlight(Transform),
    HighLightPathFinding(Pos),
    Upgrade(TowerTypes, Transform),
    Build(TowerTypes, Transform),
    DeHighlight(Transform),
}

#[derive(Debug, Default, Resource)]
pub(crate) struct HighlightedSpot(pub(crate) Option<(Entity, Transform)>);

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePosition::default())
            .insert_resource(HighlightedSpot::default())
            .insert_resource(GridResource::default())
            .add_event::<MouseClickEvent>()
            .add_event::<GridClickEvent>()
            .add_systems(Update, track_mouse_position_system)
            .add_systems(Update, mouse_clicked)
            .add_systems(Update, highight)
            .add_systems(Update, dehighlight)
            .add_systems(Startup, setup);
    }
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
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    grid: Res<GridResource>,
    window: Query<&Window>,
    mut mouse_writer: EventWriter<MouseClickEvent>,
) {
    let screen_size = get_screen_size(window);
    for b in mouse_button_input.get_just_pressed() {
        let translated_mouse_position =
            translate_mouse_coords(mouse_position.position, screen_size);
        let grid_coords = normalize_to_grid(translated_mouse_position, &grid);

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
                button: *b,
            });
        }
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

            let trx = Transform::from_xyz(grid.top_right().0, grid.top_right().1, 0.0);

            // Create me a sprite here

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..Default::default()
                },
                transform: trx,
                ..Default::default()
            });
        }

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
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
    grid.set_grid()
}

fn highight(
    mut commands: Commands,
    mut grid_click_events: EventReader<GridClickEvent>,
    mut highlighted_spot: ResMut<HighlightedSpot>,
    grid: Res<GridResource>,
) {
    for event in grid_click_events.read() {
        match event {
            GridClickEvent::Highlight(transform) => {
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
                highlighted_spot.0 = Some((entity, offset_transform));
            }

            GridClickEvent::HighLightPathFinding(pos) => {
                let grid_square_size = grid.get_grid_square_size();
                let x = pos.x() as f32 * grid_square_size + grid.bottom_left().0;
                let y = pos.y() as f32 * grid_square_size + grid.bottom_left().1;

                let dot_size = grid_square_size / 10.0; // Size of each dot
                let num_dots = (grid_square_size / (dot_size * 2.0)).floor() as usize; // Number of dots

                for i in 0..num_dots {
                    let offset_x = i as f32 * dot_size * 2.0; // Position of the dot

                    let transform = Transform::from_xyz(x + offset_x, y, 0.0);

                    let offset_transform = Transform {
                        translation: Vec3::new(
                            transform.translation.x + dot_size / 2.0,
                            transform.translation.y + grid_square_size / 2.0,
                            transform.translation.z,
                        ),
                        ..transform
                    };
                    let entity = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::BISQUE,
                                custom_size: Some(Vec2::splat(dot_size)),
                                ..Default::default()
                            },
                            transform: offset_transform,
                            ..Default::default()
                        })
                        .id();
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
) {
    for event in grid_click_events.read() {
        match event {
            GridClickEvent::DeHighlight(_transform) => {
                if let Some(entity) = highlighted_spot.0 {
                    let maybe_entity = commands.get_entity(entity.0);
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
