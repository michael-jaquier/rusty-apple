//! Mobs

use std::{collections::HashMap, hash::Hash, sync::atomic::AtomicUsize};

use enum_iterator::Sequence;

use crate::{assets::SpriteAssets, prelude::*};

static SPAWNER_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) mod enemy;
pub use enemy::MobPlugin;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Sequence)]
pub(crate) enum Enemies {
    Block,
}

impl Enemies {
    pub(crate) fn damage(&self, map_level: u32) -> u32 {
        match self {
            Enemies::Block => (1 + map_level).min(20),
        }
    }
}

fn block_enemy_sprite() -> Sprite {
    Sprite {
        color: Color::CRIMSON,
        custom_size: Some(Vec2::new(16.0, 16.0)),
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StatusEffect {
    pub(crate) effect_type: EffectType,
    pub(crate) timer: Timer,
    pub(crate) potency: u32,
}

impl Hash for StatusEffect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.effect_type.hash(state);
    }
}

impl PartialEq for StatusEffect {
    fn eq(&self, other: &Self) -> bool {
        self.effect_type == other.effect_type
    }
}

impl Eq for StatusEffect {}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Copy)]
pub(crate) enum EffectType {
    None,
    Slow,
    // add other effect types here
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
    pub(crate) next_position: Option<Vec3>,
    pub(crate) move_speed: f32,
    pub(crate) experience: usize,
    pub(crate) bricks: usize,
    pub(crate) status_effects: HashMap<EffectType, StatusEffect>,
}

impl EnemyUnit {
    pub(crate) fn insert_status(&mut self, effect: StatusEffect) {
        if effect.effect_type == EffectType::None {
            return;
        }
        self.status_effects.insert(effect.effect_type, effect);
    }
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
    pub(crate) max_kill: usize,
    pub(crate) current_kill: usize,
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
            next_position: None,
            move_speed: 30.0 + (map_level as f32 * 0.03).min(190.0),
            experience: 1,
            bricks: 1,
            status_effects: HashMap::new(),
        };

        match self {
            Enemies::Block => {
                base.health = 3 * (map_level.pow(2) as usize);
                base.experience += map_level as usize / 3;
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
