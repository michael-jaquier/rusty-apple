#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use std::marker::PhantomData;

use bevy::{
    animation,
    input::mouse::{self, MouseButtonInput},
    prelude::*,
    time,
    window::WindowResolution,
};
fn main() {
    App::new()
        .insert_resource(LaserTracker::default())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, sprite_movement)
        .add_systems(Update, mouse_events)
        .add_systems(Update, shoot_laser)
        .add_systems(Update, animate_laser)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("link.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 48.0), 12, 8, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation

    let rc = |row: usize, column, start, end| {
        let mut indices = Vec::new();
        for i in start..=end {
            indices.push(row * 12 + column + i);
        }
        indices
    };
    let rows_columns = rc(6, 9, 0, 2);
    let animation_indices = AnimationIndices::new(rows_columns[0], *rows_columns.last().unwrap());
    let mouse_sprite = AnimationIndices::new(rows_columns[0], *rows_columns.last().unwrap());
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(2.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        SpriteType::<AutoSprite>::default(),
    ));

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(mouse_sprite.first),
            transform: Transform::from_scale(Vec3::splat(3.0)),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        mouse_sprite,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        SpriteType::<MouseCursor>::default(),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &SpriteType<AutoSprite>,
    )>,
) {
    for (mut indices, mut timer, mut sprite, _auto) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let sidx = indices.next().unwrap();
            sprite.index = sidx;
        }
    }
}

fn sprite_movement(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &mut AnimationIndices,
        &SpriteType<AutoSprite>,
    )>,
    mut window: Query<&mut Window>,
) {
    for (mut transform, indices, _) in &mut query {
        let window = window.single_mut();
        let current_window_size = window.width();
        transform.translation.x += (indices.current as f32 - 6.0) * time.delta_seconds();

        if transform.translation.x > current_window_size / 2.0 {
            transform.translation.x = -current_window_size / 2.0;
        }
    }
}

#[derive(Resource)]
struct LaserTracker {
    count: usize,
    timer: Timer,
}

impl Default for LaserTracker {
    fn default() -> Self {
        Self {
            count: 0,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

fn shoot_laser(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Visibility, &SpriteType<MouseCursor>)>,
    mut laser_tracker: ResMut<LaserTracker>,
    asset_server: Res<AssetServer>,
) {
    laser_tracker.timer.tick(time.delta());
    if laser_tracker.timer.finished() && laser_tracker.count <= 1 {
        for (visibility, _) in &mut query {
            if *visibility == Visibility::Visible {
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("laser.png"),
                        transform: Transform::from_scale(Vec3::splat(0.2)),
                        ..default()
                    },
                    SpriteType::<WeaponEffect>::default(),
                ));
                laser_tracker.count += 1;
            }
        }
    }
}

fn animate_laser(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &Sprite,
        Option<&Handle<Image>>,
        &SpriteType<WeaponEffect>,
    )>,

    mut windows: Query<&mut Window>,
    mut laser_tracker: ResMut<LaserTracker>,
    images: Res<Assets<Image>>,
) {
    let window_width = windows.single_mut().width();
    for (entity, mut transform, sprite, maybe_handle, _) in &mut query {
        transform.translation.x += 300.0 * time.delta_seconds();
        let size = if let Some(custom_size) = sprite.custom_size {
            custom_size
        } else if let Some(image) = maybe_handle.and_then(|handle| images.get(handle)) {
            Vec2::new(image.width() as f32, image.height() as f32)
        } else {
            Vec2::new(1.0, 1.0)
        };
        if transform.translation.x > (window_width + size.x / 2.0) / 2.0 {
            commands.entity(entity).despawn();
            laser_tracker.count -= 1;
        }
    }
}

fn mouse_events(
    mut query: Query<(
        &mut Transform,
        &mut AnimationIndices,
        &mut Visibility,
        &SpriteType<MouseCursor>,
    )>,
    mut cursor_events: EventReader<CursorMoved>,
    mut click_events: EventReader<MouseButtonInput>,
    mut windows: Query<&mut Window>,
) {
    for event in click_events.read() {
        for (_, _, mut visibility, _) in &mut query {
            match event.button {
                MouseButton::Left => *visibility = Visibility::Visible,
                MouseButton::Right => *visibility = Visibility::Hidden,
                MouseButton::Middle => {}
                MouseButton::Other(_) => {}
            }
        }
    }

    for event in cursor_events.read() {
        for (mut transform, mut indices, _, _) in &mut query {
            let window = windows.single_mut();
            let current_window_size = window.width();
            let current_window_height = window.height();

            let x = event.position.x;
            let y = event.position.y;
            transform.translation.x = x - current_window_size / 2.0;
            transform.translation.y = current_window_height / 2.0 - y;
            indices.current = 0;
        }
    }
}

#[derive(Component)]
struct SpriteType<T> {
    _phantom: PhantomData<T>,
}

impl<T> Default for SpriteType<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[derive(Component)]
struct MouseCursor;

#[derive(Component)]
struct AutoSprite;

#[derive(Component)]
struct WeaponEffect;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
    current: usize,
}

impl AnimationIndices {
    fn new(first: usize, last: usize) -> Self {
        Self {
            first,
            last,
            current: first,
        }
    }
}

impl Iterator for AnimationIndices {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.last {
            self.current = self.first;
            Some(self.current)
        } else {
            let current = self.current;
            self.current += 1;
            Some(current)
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);
