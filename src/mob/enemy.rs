//! Enemies

use crate::{assets::SpriteAssets, prelude::*};
use enum_iterator::Sequence;

use super::{MobSpawner, MobSpawnerData, SpawnId};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Sequence)]
pub(crate) enum Enemies {
    Block,
}

impl Enemies {
    pub(crate) fn into_unit(&self, id: SpawnId) -> EnemyUnit {
        let mut base = EnemyUnit {
            mob_type: *self,
            spwawner_id: id,
            health: 1,
        };

        match self {
            Enemies::Block => {
                base.health = 3;
            }
        }

        base
    }
    pub(crate) fn points(&self) -> usize {
        match self {
            Enemies::Block => 1,
        }
    }
}

impl From<Enemies> for EnemyComponent {
    fn from(value: Enemies) -> Self {
        let data: MobSpawnerData = value.into();
        let spawner: MobSpawner = data.into();
        EnemyComponent {
            mob_type: value,
            spawner,
        }
    }
}

impl From<Enemies> for MobSpawnerData {
    fn from(value: Enemies) -> Self {
        let (period, count) = match value {
            Enemies::Block => (5.0, 1),
        };

        MobSpawnerData {
            mob_type: value,
            spawn_position: Position::from_xy(0.0, 0.0),
            period: period,
            max_count: count,
        }
    }
}

impl Enemies {
    pub(crate) fn set(assets: &mut SpriteAssets) {
        assets
            .enemy_sprites
            .insert(Enemies::Block, block_enemy_sprite());
    }
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

#[derive(Debug, Component, Copy, Clone)]
pub(crate) struct EnemyUnit {
    pub(crate) mob_type: Enemies,
    pub(crate) spwawner_id: SpawnId,
    pub(crate) health: usize,
}
