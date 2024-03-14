//! Path Finding A*

use std::collections::{BinaryHeap, HashMap};

use bevy::prelude::*;
use bevy_xpbd_2d::plugins::collision::Collider;
use pathfinding::directed::bfs::bfs;

use super::grid::{GridClickEvent, GridResource};

/// A* path finding algorithm plugin.
pub struct PathFindingPlugin;

impl Plugin for PathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, path_finding_system);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
}

/// Breadth first search algorithm.
fn path_finding_system(
    grid: Res<GridResource>,
    mut click_event_writer: EventWriter<GridClickEvent>,
) {
    let start = Pos::new(10, 5);
    let goal = Pos::new(0, 5);
    let result = bfs(&start, |p| grid.successors(p), |p| *p == goal);
    if let Some(pos_vector) = result {
        for pos in pos_vector {
            click_event_writer.send(GridClickEvent::HighLightPathFinding(pos));
        }
    }
}
