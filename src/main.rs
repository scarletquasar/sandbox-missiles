pub mod hud;
mod player;
mod scenes;
mod sprites;

use bevy::{asset::AssetPlugin, prelude::*};

use crate::hud::display_hud;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.54, 0.62, 0.39)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player::move_player,
                sprites::hero_tileset::execute_hero_animations,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Instances camera, hud and player
    commands.spawn(Camera2d);
    let scene = scenes::spawn_scene(
        &mut commands,
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
        scenes::SceneName::Main,
        None,
    );

    let hero = sprites::hero_tileset::spawn_hero(
        &mut commands,
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
        scene.player_spawn,
        sprites::hero_tileset::HeroDirection::Front,
    );
    commands.entity(hero).insert(player::Player::default());

    display_hud(
        &mut commands,
        &hud::HudData {
            health: 100,
            energy: 50,
        },
    );
}
