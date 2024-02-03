//! Player module
use crate::{
    assets::SpriteAssets,
    prelude::*,
    weapon::{FireWeaponEvent, WeaponComponent, WeaponTypes},
};

/// The player action enum.
#[derive(Actionlike, Reflect, PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub enum PlayerAction {
    /// The fire action.
    Fire,
}

/// The player component.
#[derive(Component, Clone, Copy)]
pub struct PlayerComponent {
    pub(crate) velocity: Vec2,
    pub(crate) health: usize,
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
            .add_systems(PostStartup, spawn_player)
            .add_systems(Update, fire_weapon_system);
    }
}

fn spawn_player(mut commands: Commands, handles: Res<SpriteAssets>) {
    let input_map = InputMap::new([(KeyCode::W, PlayerAction::Fire)]);
    let mut player_entity = commands.spawn_empty();
    player_entity.insert(SpriteBundle {
        sprite: Sprite { ..default() },
        transform: Transform {
            translation: Vec3::new(-400.0, 0.0, 1.0),
            scale: Vec3::splat(0.42),
            ..default()
        },
        texture: handles.player.clone(),
        ..default()
    });

    player_entity.insert(PlayerComponent {
        velocity: Vec2::ZERO,
        health: 3,
    });

    player_entity.insert(RigidBody::Static);
    player_entity.insert(Collider::cuboid(30.0, 20.0));
    player_entity.insert(ExternalForce::ZERO);
    player_entity.insert(LinearVelocity::ZERO);
    player_entity.insert(AngularVelocity::ZERO);
    player_entity.insert(InputManagerBundle::<PlayerAction> {
        action_state: ActionState::default(),
        input_map,
    });
    player_entity.insert(WeaponComponent::from(WeaponTypes::Laser));
}

fn fire_weapon_system(
    mut player_query: Query<
        (
            &mut WeaponComponent,
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
    for (mut weapon, transform, _, _, action_state, entity) in player_query.iter_mut() {
        let fire = action_state.just_pressed(PlayerAction::Fire);

        if !fire {
            continue;
        }
        if let Some(weapon_projectile_data) = weapon.fire() {
            fire_weapon_events.send(FireWeaponEvent {
                weapon_projectile_data: weapon_projectile_data,
                source_transform: *transform,
                source_entity: entity,
                velocity: LinearVelocity(Vec2::new(300.0, 0.0)),
            });
        }
    }
}
