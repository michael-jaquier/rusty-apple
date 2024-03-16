//! Path Finding A*

use std::{
    collections::{BinaryHeap, HashMap, VecDeque},
    path,
    thread::current,
};

use bevy::{prelude::*, transform::commands};
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
    CurrentPath(Vec<Pos>, Vec<Transform>),
    HighlightCurrentPath(Pos, Vec<Pos>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub(crate) struct Pos(usize, usize);

impl Pos {
    pub(crate) fn x(&self) -> usize {
        self.0
    }
    pub(crate) fn y(&self) -> usize {
        self.1
    }
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self(x, y)
    }

    pub(crate) fn to_transform(&self, grid_square_size: f32, bottom_left: (f32, f32)) -> Transform {
        let x = self.0 as f32 * grid_square_size + bottom_left.0;
        let y = self.1 as f32 * grid_square_size + bottom_left.1;
        let offset = grid_square_size / 2.0;

        Transform {
            translation: Vec3::new(x + offset, y + offset, 0.0),
            ..Default::default()
        }
    }

    pub(crate) fn to_position(&self, grid_square_size: f32, bottom_left: (f32, f32)) -> Position {
        let x = self.0 as f32 * grid_square_size + bottom_left.0;
        let y = self.1 as f32 * grid_square_size + bottom_left.1;
        let offset = grid_square_size / 2.0;

        Position::from_xy(x + offset, y + offset)
    }

    pub(crate) fn from_transform(
        transform: &Transform,
        grid_square_size: f32,
        bottom_left: (f32, f32),
    ) -> Self {
        let x = ((transform.translation.x - bottom_left.0) / grid_square_size) as usize;
        let y = ((transform.translation.y - bottom_left.1) / grid_square_size) as usize;
        Self(x, y)
    }

    pub(crate) fn translation_difference(&self, other: &Pos, grid_square_size: f32) -> Vec3 {
        let x = (self.0 as f32 - other.0 as f32) * grid_square_size;
        let y = (self.1 as f32 - other.1 as f32) * grid_square_size;
        Vec3::new(x, y, 0.0)
    }
}

pub(crate) fn path_finding(grid: &GridResource, current: Pos) -> Option<Vec<Pos>> {
    bfs(
        &current,
        |p| grid.successors(p),
        |p| p == &grid.grid_enemy_end,
    )
}

pub(crate) fn path_mob_finding(grid: &GridResource, current: Pos) -> Option<Pos> {
    bfs(
        &current,
        |p| grid.successors(p),
        |p| p == &grid.grid_enemy_end,
    )
    .and_then(|p| p.get(1).cloned())
}

fn path(grid: Res<GridResource>, mut path_event_writer: EventWriter<PathFindingEvent>) {
    let path = path_finding(&grid, grid.grid_enemy_start);
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
