//! Arena module.

use crate::prelude::*;

/// The arena module contains the arena plugin and the arena component.
pub const ARENA_WIDTH: f32 = 1280.0;
/// The arena height.
pub const ARENA_HEIGHT: f32 = 800.0;

/// The grid square size.
pub const GRID_SQUARE_SIZE: f32 = 50.0;
pub mod grid;
pub mod path_finding;

pub use grid::GridPlugin;
pub use path_finding::PathFindingPlugin;
