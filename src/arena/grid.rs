//! Grid

use bevy::{input::mouse, prelude::*, sprite::Anchor};
use bevy_xpbd_2d::parry::na::ComplexField;
use leafwing_input_manager::user_input::InputKind;

use crate::weapon::WeaponTypes;

use super::{ARENA_HEIGHT, ARENA_WIDTH};
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

#[derive(Debug, Event)]
pub(crate) enum GridClickEvent {
    Highlight(Transform),
    Upgrade(WeaponTypes, Transform),
    Build(WeaponTypes, Transform),
}

#[derive(Debug, Default, Resource)]
pub(crate) struct HighlightedSpot(pub(crate) Option<(Entity, Transform)>);

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePosition::default())
            .insert_resource(HighlightedSpot::default())
            .add_event::<MouseClickEvent>()
            .add_event::<GridClickEvent>()
            .add_systems(Update, track_mouse_position_system)
            .add_systems(Update, mouse_clicked)
            .add_systems(Update, highight)
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

fn mouse_clicked(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut mouse_writer: EventWriter<MouseClickEvent>,
) {
    for b in mouse_button_input.get_just_pressed() {
        let position = mouse_position.position;
        let square_size = 50.0; // Size of each square
        let x = (position.x / square_size).floor() * square_size + square_size / 2.0;
        let y = ARENA_HEIGHT - (position.y / square_size).floor() * square_size - square_size / 2.0;
        let transform = Transform::from_xyz(x - ARENA_WIDTH / 2.0, y - ARENA_HEIGHT / 2.0, 0.0);
        mouse_writer.send(MouseClickEvent {
            position: transform,
            button: *b,
        });
    }
}

fn setup(mut commands: Commands) {
    let square_size = 50.0; // Size of each square
    let squares_x = (ARENA_WIDTH / square_size).ceil() as i32;
    let squares_y = (ARENA_HEIGHT / square_size).ceil() as i32;

    let line_thickness = 1.0; // Thickness of the lines

    for y in 0..=squares_y {
        let line_transform =
            Transform::from_xyz(0.0, y as f32 * square_size - ARENA_HEIGHT / 2.0, 0.0);
        println!("line_transform: {:?}", line_transform.translation.y);

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(ARENA_WIDTH, line_thickness)),
                ..Default::default()
            },
            transform: line_transform,
            ..Default::default()
        });
    }
    // Create the vertical lines
    for x in 0..=squares_x {
        let line_transform =
            Transform::from_xyz(x as f32 * square_size - ARENA_WIDTH / 2.0, 0.0, 0.0);

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(line_thickness, ARENA_HEIGHT)),
                ..Default::default()
            },
            transform: line_transform,
            ..Default::default()
        });
    }
}

fn highight(
    mut commands: Commands,
    mut grid_click_events: EventReader<GridClickEvent>,
    mut highlighted_spot: ResMut<HighlightedSpot>,
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
                let entity = commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            custom_size: Some(Vec2::new(50.0, 50.0)),
                            ..Default::default()
                        },
                        transform: *transform,
                        ..Default::default()
                    })
                    .id();
                highlighted_spot.0 = Some((entity, *transform));
            }
            _ => {}
        }
    }
}
