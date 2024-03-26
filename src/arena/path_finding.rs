//! Path Finding A*

use std::{
    collections::{BinaryHeap, HashMap, VecDeque},
    path,
    thread::current,
};

use bevy::{prelude::*, transform::commands};
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_egui::egui::debug_text::print;
use bevy_xpbd_2d::components::Position;
use pathfinding::directed::bfs::bfs;

use super::grid::{GridClickEvent, GridResource};

/// A* path finding algorithm plugin.
pub struct PathFindingPlugin;

impl Plugin for PathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PathFindingEvent>();
    }
}

#[derive(Debug, Clone, Event)]
pub(crate) struct NoPathEvent;

#[derive(Debug, Clone, Event)]
pub(crate) enum PathFindingEvent {
    NewObstacle,
    NoPath(Entity),
    CurrentPath(Vec<TilePos>, Vec<Transform>),
    HighlightCurrentPath(TilePos, Vec<TilePos>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub(crate) struct Pos(usize, usize);

impl From<Pos> for TilePos {
    fn from(pos: Pos) -> Self {
        TilePos {
            x: pos.0 as u32,
            y: pos.1 as u32,
        }
    }
}
pub(crate) fn to_transform(tile_postition: TilePos, grid: &GridResource) -> Transform {
    let transform = grid.grid_transform[tile_postition.x as usize][tile_postition.y as usize];
    Transform {
        translation: Vec3::new(transform.x, transform.y, 0.0),
        ..Default::default()
    }
}

pub(crate) fn to_position(tile_position: TilePos, grid: &GridResource) -> Position {
    let transform = to_transform(tile_position, grid).translation.truncate();
    Position(transform)
}

pub(crate) fn from_transform(
    transform: &Transform,
    grid_square_size: f32,
    bottom_left: (f32, f32),
) -> TilePos {
    let x = ((transform.translation.x - bottom_left.0) / grid_square_size) as u32;
    let y = ((transform.translation.y - bottom_left.1) / grid_square_size) as u32;
    TilePos { x, y }
}

pub(crate) fn path_finding(grid: &GridResource, current: TilePos) -> Option<Vec<TilePos>> {
    bfs(
        &current,
        |p| grid.successors(p),
        |p| p == &grid.grid_enemy_end,
    )
}

pub(crate) fn path_mob_finding(grid: &GridResource, current: TilePos) -> Option<TilePos> {
    let x = bfs(
        &current,
        |p| grid.successors(p),
        |p| p == &grid.grid_enemy_end,
    );
    x.and_then(|p| p.get(1).cloned())
}

fn path(grid: Res<GridResource>, mut path_event_writer: EventWriter<PathFindingEvent>) {
    let path = path_finding(&grid, grid.grid_enemy_start.into());
    match path {
        Some(p) => {
            path_event_writer.send(PathFindingEvent::HighlightCurrentPath(
                *p.first().unwrap(),
                p,
            ));
        }
        _ => {}
    }
}
