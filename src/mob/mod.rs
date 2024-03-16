//! Mobs

use std::sync::atomic::AtomicUsize;

use enum_iterator::{all, Sequence};

use crate::{
    arena::{
        grid::GridResource,
        path_finding::{path_mob_finding, Pos},
        ARENA_HEIGHT, ARENA_WIDTH,
    },
    assets::SpriteAssets,
    collision::GameLayer,
    prelude::*,
    ui::level::MapLevel,
};

static SPAWNER_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) mod enemy;
pub use enemy::MobPlugin;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Sequence)]
pub(crate) enum Enemies {
    Block,
}

fn block_enemy_sprite() -> Sprite {
    Sprite {
        color: Color::rgb(0.0, 1.0, 0.0),
        custom_size: Some(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }
}

/// The enemy component.
#[derive(Component, Debug)]
pub(crate) struct EnemyComponent {
    pub(crate) mob_type: Enemies,
    pub(crate) spawner: MobSpawner,
}

#[derive(Debug, Component, Clone)]
pub(crate) struct EnemyUnit {
    pub(crate) mob_type: Enemies,
    pub(crate) spwawner_id: SpawnId,
    pub(crate) health: usize,
    pub(crate) move_timer: Timer,
}
#[derive(Debug)]
pub(crate) struct MobSpawnerData {
    pub(crate) mob_type: Enemies,
    pub(crate) spawn_position: Position,
    pub(crate) period: f32,
    pub(crate) max_count: usize,
}

#[derive(Debug)]
pub(crate) struct MobSpawner {
    mob_type: Enemies,
    spawn_position: Position,
    timer: Timer,
    max_count: usize,
    current_count: usize,
    max_kill: usize,
    current_kill: usize,
    spawner_id: SpawnId,
}

impl From<MobSpawnerData> for MobSpawner {
    fn from(data: MobSpawnerData) -> Self {
        MobSpawner {
            mob_type: data.mob_type,
            spawn_position: data.spawn_position,
            timer: Timer::from_seconds(data.period, TimerMode::Repeating),
            max_count: data.max_count,
            current_count: 0,
            max_kill: 0,
            current_kill: 0,
            spawner_id: SpawnId::new(),
        }
    }
}

#[derive(Debug, Component, Clone, Copy, Eq, PartialEq)]
pub(crate) struct SpawnId {
    pub(crate) id: usize,
}

impl SpawnId {
    pub(crate) fn new() -> Self {
        SpawnId {
            id: SPAWNER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Event)]
pub(crate) struct MobSpawnEvent {
    pub(crate) mob_type: Enemies,
    pub(crate) position: Position,
    pub(crate) spawner_id: SpawnId,
    pub(crate) map_level: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum EnemyDespawnReason {
    Killed,
    ReachedEnd,
}
#[derive(Debug, Event)]
pub(crate) struct MobDespawnEvent {
    pub(crate) enemy_entity: Entity,
    pub(crate) spawner_id: SpawnId,
    pub(crate) reason: EnemyDespawnReason,
}

impl Enemies {
    pub(crate) fn into_unit(&self, id: SpawnId, map_level: u32) -> EnemyUnit {
        let mut base = EnemyUnit {
            mob_type: *self,
            spwawner_id: id,
            health: 1,
            move_timer: Timer::from_seconds(0.5, TimerMode::Once),
        };

        match self {
            Enemies::Block => {
                base.health = 3 * (map_level as usize);
            }
        }

        base
    }
}

impl Enemies {
    pub(crate) fn set(assets: &mut SpriteAssets) {
        assets
            .enemy_sprites
            .insert(Enemies::Block, block_enemy_sprite());
    }
}
