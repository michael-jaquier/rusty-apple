//! Path Finding A*

use std::collections::{BinaryHeap, HashMap, VecDeque};

use bevy::prelude::*;
use pathfinding::directed::bfs::bfs;

use super::grid::{GridClickEvent, GridResource};

/// A* path finding algorithm plugin.
pub struct PathFindingPlugin;

impl Plugin for PathFindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, path);
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
fn better_path_finding_system(
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

fn path(
    time: Res<Time>,
    grid: Res<GridResource>,
    mut click_event_writer: EventWriter<GridClickEvent>,
) {
    let start = Pos::new(10, 5);
    let goal = Pos::new(0, 5);
    let mut visited = HashMap::new();
    visited.insert(start, None);
    let mut queue = VecDeque::new();
    queue.push_front(start);

    while let Some(current_node) = queue.pop_back() {
        if timer.finished() {
            timer.reset();
            let next_nodes = grid.successors(&current_node);
            for next_node in next_nodes {
                if !visited.contains_key(&next_node) {
                    visited.insert(next_node, Some(current_node));
                    queue.push_front(next_node);
                }
            }

            let current_head = current_node;

            let mut current_drawing = Vec::new();
            let mut pt = Some(current_node);
            while let Some(path_segment) = pt {
                current_drawing.push(path_segment);
                pt = *visited.get(&path_segment).unwrap_or(&None);
            }

            click_event_writer.send(GridClickEvent::HighlightCurrentPath(
                current_head,
                current_drawing,
            ));

            if current_node == goal {
                break;
            }
        }
    }
}
