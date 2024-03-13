//! Mobs

use std::sync::atomic::AtomicUsize;

use bevy::{asset, transform::commands, utils::tracing::Instrument};
use bevy_xpbd_2d::parry::na::coordinates::X;
use enum_iterator::all;
use rand::Rng;

use crate::{
    arena::{ARENA_HEIGHT, ARENA_WIDTH},
    assets::SpriteAssets,
    prelude::*,
};

static SPAWNER_ID: AtomicUsize = AtomicUsize::new(0);

use self::enemy::{EnemyComponent, EnemyUnit};

pub(crate) mod enemy;

#[derive(Debug)]
pub(crate) struct MobSpawnerData {
    pub(crate) mob_type: enemy::Enemies,
    pub(crate) spawn_position: Position,
    pub(crate) period: f32,
    pub(crate) max_count: usize,
}

#[derive(Debug)]
pub(crate) struct MobSpawner {
    mob_type: enemy::Enemies,
    spawn_position: Position,
    timer: Timer,
    max_count: usize,
    current_count: usize,
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

    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn is(&self, other: &SpawnId) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Event)]
pub(crate) struct MobSpawnEvent {
    pub(crate) mob_type: enemy::Enemies,
    pub(crate) position: Position,
    pub(crate) spawner_id: SpawnId,
}

#[derive(Debug, Event)]
pub(crate) struct MobDespawnEvent {
    pub(crate) enemy_entity: Entity,
    pub(crate) spawner_id: SpawnId,
}

/// The mob plugin.
pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MobSpawnEvent>()
            .add_event::<MobDespawnEvent>()
            .add_systems(PostStartup, deploy_mod_spawners)
            .add_systems(Update, spawn_enemy)
            .add_systems(Update, mob_spawn_system)
            .add_systems(Update, (trigger_despawn_event, mob_despawn_system).chain());
    }
}

fn mob_spawn_system(
    mut event: EventWriter<MobSpawnEvent>,
    mut mob_query: Query<&mut EnemyComponent>,
    time: Res<Time>,
) {
    for (mut enemy) in mob_query.iter_mut() {
        if enemy.spawner.timer.tick(time.delta()).just_finished()
            && enemy.spawner.current_count < enemy.spawner.max_count
        {
            event.send(MobSpawnEvent {
                mob_type: enemy.mob_type,
                position: enemy.spawner.spawn_position,
                spawner_id: enemy.spawner.spawner_id,
            });
        }

        enemy.spawner.current_count += 1;
    }
}

fn deploy_mod_spawners(mut commands: Commands) {
    for enemy in all::<enemy::Enemies>() {
        let enemy: enemy::EnemyComponent = enemy.into();
        let mut entity = commands.spawn_empty();
        entity.insert(enemy);
    }
}

fn spawn_enemy(
    mut commands: Commands,
    assets: Res<SpriteAssets>,
    mut event: EventReader<MobSpawnEvent>,
) {
    for mob_spawn_event in event.read() {
        let sprite = assets.enemy_sprites[&mob_spawn_event.mob_type].clone();

        commands.spawn((
            SpriteBundle {
                sprite,
                transform: Transform {
                    translation: Vec3::Z * 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            mob_spawn_event.position,
            Collider::rectangle(50.0, 50.0),
            RigidBody::Kinematic,
            ExternalForce::ZERO,
            LinearVelocity::ZERO,
            AngularVelocity(0.0),
            mob_spawn_event
                .mob_type
                .into_unit(mob_spawn_event.spawner_id),
        ));
    }
}

fn mob_despawn_system(
    mut commands: Commands,
    mut event: EventReader<MobDespawnEvent>,
    mut query: Query<&mut EnemyComponent>,
) {
    for (mob_despawn_event, id) in event.read_with_id() {
        let maybe_entity = commands.get_entity(mob_despawn_event.enemy_entity);
        if let Some(mut entity) = maybe_entity {
            // get the enemy component that matches our id
            for mut enemy in query.iter_mut() {
                if enemy.spawner.spawner_id == mob_despawn_event.spawner_id {
                    enemy.spawner.current_count = enemy.spawner.current_count.saturating_sub(1);
                }
            }

            entity.despawn()
        }
    }
}

fn trigger_despawn_event(
    mut event: EventWriter<MobDespawnEvent>,
    query: Query<(Entity, &EnemyUnit, &Position)>,
) {
    // Count the number of enemies in the arena
    let count = query.iter().count();
    if count > 0 {
        for (entity, enemy, position) in query.iter() {
            if position.x.abs() > ARENA_WIDTH as f32 || position.y.abs() > ARENA_HEIGHT as f32 {
                event.send(MobDespawnEvent {
                    enemy_entity: entity,
                    spawner_id: enemy.spwawner_id,
                });
            }
        }
    }
}
