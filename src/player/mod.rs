//! Player module.

use crate::{prelude::*, ui::level::LevelMap};

/// The player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerUpdateEvent>()
            .add_systems(Startup, create)
            .add_systems(Update, update)
            .add_systems(Update, level_down_on_death);
    }
}

// Player Components
#[derive(Debug, Clone, Component)]
pub(crate) struct Player {
    pub(crate) hp: u32,
    pub(crate) bricks: u32,
}

impl Default for Player {
    fn default() -> Self {
        Player { hp: 100, bricks: 5 }
    }
}

impl Player {
    pub(crate) fn remove_bricks(&mut self, amount: u32) -> bool {
        if amount > self.bricks {
            false
        } else {
            self.bricks = self.bricks.saturating_sub(amount);
            true
        }
    }
}

#[derive(Debug, Clone, Event)]
pub(crate) enum PlayerUpdateEvent {
    Damage(u32),
    Bricks(u32),
    Build(u32),
}

fn create(mut commands: Commands) {
    commands.spawn(Player::default());
}

fn update(mut player_update_event: EventReader<PlayerUpdateEvent>, mut player: Query<&mut Player>) {
    let mut player = player.single_mut();
    for event in player_update_event.read() {
        match event {
            PlayerUpdateEvent::Damage(damage) => player.hp = player.hp.saturating_sub(*damage),
            PlayerUpdateEvent::Bricks(bricks) => {
                player.bricks = player.bricks.saturating_add(*bricks)
            }
            PlayerUpdateEvent::Build(bricks) => {
                if player.bricks >= *bricks {
                    player.bricks = player.bricks.saturating_sub(*bricks);
                } else {
                    warn!(
                        "Not enough bricks to build this should not happen!
                    Cost {} Bricks, Player has {} Bricks",
                        bricks, player.bricks
                    );
                }
            }
        }
    }
}

fn level_down_on_death(mut player: Query<&mut Player>, mut level_up_map: EventWriter<LevelMap>) {
    // Could be faster to use a filter_map here or an event
    let mut player = player.single_mut();
    if player.hp == 0 {
        level_up_map.send(LevelMap::LevelDown(5));
        player.hp = 100;
    }
}
