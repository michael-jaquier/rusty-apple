//! # Assets

use bevy::utils::HashMap;

use crate::{prelude::*, weapon::WeaponTypes};

/// The assets plugin.
#[derive(Resource)]
pub struct SpriteAssets {
    /// The player sprite.
    pub player: Handle<Image>,
    pub(crate) weapon_sprites: HashMap<WeaponTypes, Sprite>,
}

/// The assets plugin.
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut _texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut sprite_assets = SpriteAssets {
        player: asset_server.load("orc.png"),
        weapon_sprites: HashMap::default(),
    };
    sprite_assets
        .weapon_sprites
        .insert(WeaponTypes::Laser, laser_sprite());
    commands.insert_resource(sprite_assets);

    // texture_atlas(
    //     texture_atlases,
    //     asset_server,
    //     "link.png",
    //     (Vec2::new(48.0, 48.0), 12, 8),
    //     "link",
    // );
}

// fn texture_atlas(
//     mut texture_atlases: ResMut<Assets<TextureAtlas>>,
//     asset_server: Res<AssetServer>,
//     path: &str,
//     grid: (Vec2, usize, usize),
//     name: &str,
// ) {
//     let texture_handle = asset_server.load(path.clone());
//     let atlas = TextureAtlas::from_grid(texture_handle, grid.0, grid.1, grid.2, None, None);
//     let handle = texture_atlases.add(atlas);
// }

fn laser_sprite() -> Sprite {
    Sprite {
        color: Color::rgb(1.0, 0.0, 0.0),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..Default::default()
    }
}
