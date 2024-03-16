//! # Assets

use bevy::{asset, render::render_resource::Texture, utils::HashMap};

use crate::{
    mob::Enemies,
    prelude::*,
    towers::{self, TowerTypes},
    weapons::weapon::WeaponTypes,
};

#[derive(Resource)]
pub(crate) struct SpriteAssets {
    /// The player sprite.
    pub(crate) player: Handle<Image>,
    pub(crate) weapon_sprites: HashMap<WeaponTypes, Sprite>,
    pub(crate) enemy_sprites: HashMap<Enemies, Sprite>,
    pub(crate) tower_sprites: HashMap<TowerTypes, Handle<Image>>,
    pub(crate) other: HashMap<Enemies, Handle<Image>>,
}
/// The assets plugin.
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn texture_atlas_setup(mut atlas: ResMut<Assets<TextureAtlasLayout>>) {
    let texture_atlas = TextureAtlasLayout::from_grid(
        Vec2::new(32.0, 32.0),
        6,
        6,
        Some(Vec2::new(10.0, 10.0)),
        None,
    );

    atlas.add(texture_atlas);
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut sprite_assets = SpriteAssets {
        player: asset_server.load("orc.png"),
        weapon_sprites: HashMap::default(),
        enemy_sprites: HashMap::default(),
        tower_sprites: HashMap::default(),
        other: HashMap::default(),
    };

    let m = asset_server.load("isometric-sheet.png");
    sprite_assets.other.insert(Enemies::Block, m);
    WeaponTypes::set(&mut sprite_assets);
    Enemies::set(&mut sprite_assets);
    towers::TowerTypes::set(&mut sprite_assets, asset_server);

    commands.insert_resource(sprite_assets);
}

#[derive(Component)]
pub(crate) struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AnimationTimer(Timer);
