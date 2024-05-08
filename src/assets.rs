//! # Assets

use bevy::utils::HashMap;

use crate::{
    mob::{self, enemy::Enemies},
    prelude::*,
    weapon::{self, WeaponTypes},
};

/// The assets plugin.
#[derive(Resource)]
pub struct SpriteAssets {
    /// The player sprite.
    pub player: Handle<Image>,
    pub(crate) weapon_sprites: HashMap<WeaponTypes, Sprite>,
    pub(crate) enemy_sprites: HashMap<Enemies, Sprite>,
}

/// The assets plugin.
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut sprite_assets = SpriteAssets {
        player: asset_server.load("orc.png"),
        weapon_sprites: HashMap::default(),
        enemy_sprites: HashMap::default(),
    };

    weapon::WeaponTypes::set(&mut sprite_assets);
    mob::enemy::Enemies::set(&mut sprite_assets);

    commands.insert_resource(sprite_assets);

}
