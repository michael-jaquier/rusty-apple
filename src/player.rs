//! Player module
use crate::{
    arena::ARENA_HEIGHT,
    assets::SpriteAssets,
    prelude::*,
    weapon::{FireWeaponEvent, WeaponComponent, WeaponComponents, WeaponTypes},
};

/// The player action enum.
#[derive(Actionlike, Reflect, PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub enum PlayerAction {
    /// The fire action.
    Fire,
}

#[derive(Debug, Clone, Event, Copy, PartialEq, Eq, Hash)]
/// Player update events
pub enum PlayerUpdateEvent {
    /// The player got score
    Score(usize),
}

impl PlayerUpdateEvent {
    /// Create a new score event
    pub fn add_points(score: usize) -> Self {
        PlayerUpdateEvent::Score(score)
    }
}

/// The player component.
#[derive(Component, Clone, Copy)]
pub struct PlayerComponent {
    pub(crate) health: usize,
    pub(crate) points: usize,
}

#[derive(Event)]
pub(crate) struct PlayerAttackContactEvent {
    pub(crate) attack: Entity,
    pub(crate) target: Entity,
}

/// The player plugin.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
        app.add_event::<PlayerAttackContactEvent>()
            .add_event::<PlayerUpdateEvent>()
            .add_systems(PostStartup, spawn_player)
            .add_systems(Update, fire_weapon_system)
            .add_systems(Update, player_update_system);
    }
}

fn spawn_player(mut commands: Commands, handles: Res<SpriteAssets>) {
    let input_map = InputMap::new([(PlayerAction::Fire, KeyCode::KeyW)]);
    let mut player_entity = commands.spawn_empty();
    player_entity.insert(SpriteBundle {
        sprite: Sprite { ..default() },
        transform: Transform {
            translation: Vec3::new(-400.0, 0.0, 1.0),
            scale: Vec3::splat(0.32),
            ..default()
        },
        texture: handles.player.clone(),
        ..default()
    });

    player_entity.insert(PlayerComponent {
        health: 3,
        points: 0,
    });

    player_entity.insert(RigidBody::Static);
    player_entity.insert(ExternalForce::ZERO);
    player_entity.insert(LinearVelocity::ZERO);
    player_entity.insert(AngularVelocity::ZERO);
    player_entity.insert(InputManagerBundle::<PlayerAction> {
        action_state: ActionState::default(),
        input_map,
    });
    let weapon_components: WeaponComponents = vec![].into();

    player_entity.insert(weapon_components);
}

fn fire_weapon_system(
    mut player_query: Query<
        (
            &mut WeaponComponents,
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &ActionState<PlayerAction>,
            Entity,
        ),
        With<PlayerComponent>,
    >,
    mut fire_weapon_events: EventWriter<FireWeaponEvent>,
) {
    for (mut weapons, transform, _, _, _, entity) in player_query.iter_mut() {
        for weapon in weapons.weapons.iter_mut() {
            if let Some(weapon_projectile_data) = weapon.fire() {
                for n in 1..=weapon_projectile_data.count {
                    // Of the count is greeter than 1, then we need to adjust the position of the projectile so they are distributed evenly
                    let spacing = 10.0;
                    let mut x = 0.0;
                    if weapon_projectile_data.count != 1 {
                        // Flip the sign if the count is even
                        if n % 2 == 0 {
                            x += -spacing * ((n.saturating_sub(1)) as f32);
                        } else {
                            x += spacing * (n as f32);
                        }
                    }

                    let mut transform = transform.clone();
                    transform.translation.y += x;

                    fire_weapon_events.send(FireWeaponEvent {
                        weapon_projectile_data: weapon_projectile_data,
                        source_transform: transform,
                        source_entity: entity,
                        velocity: LinearVelocity(Vec2::new(300.0, 0.0)),
                    });
                }
            }
        }
    }
}

fn player_update_system(
    mut player_query: Query<&mut PlayerComponent>,
    mut player_update_event: EventReader<PlayerUpdateEvent>,
) {
    for event in player_update_event.read() {
        for mut player in player_query.iter_mut() {
            match event {
                PlayerUpdateEvent::Score(score) => {
                    player.points += score;
                }
            }
        }
    }
}
