pub mod hud;
mod player;
mod scenes;
mod sprites;

use bevy::{asset::AssetPlugin, prelude::*};

use crate::hud::display_hud;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<scenes::CurrentScene>()
        .init_resource::<scenes::LevelRegistry>()
        .init_resource::<player::inventory::ItemCatalog>()
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
                scenes::reload_levels_during_runtime,
                sprites::hero_tileset::execute_hero_animations,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_scene: ResMut<scenes::CurrentScene>,
    mut level_registry: ResMut<scenes::LevelRegistry>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Instances camera, hud and player
    commands.spawn(Camera2d);
    let report = level_registry.reload_from_disk();
    for error in &report.errors {
        warn!("{error}");
    }

    let startup_level_name = if level_registry.get("main").is_some() {
        "main".to_owned()
    } else {
        level_registry
            .first_level_name()
            .map(str::to_owned)
            .expect("No levels were found in assets/levels")
    };

    let scene = scenes::spawn_scene(
        &mut commands,
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
        level_registry.as_ref(),
        &startup_level_name,
        current_scene.take(),
    )
    .expect("Failed to spawn startup level");
    let player_spawn = scene.player_spawn;
    current_scene.replace(scene);

    let hero = sprites::hero_tileset::spawn_hero(
        &mut commands,
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
        player_spawn,
        sprites::hero_tileset::HeroDirection::Front,
    );
    commands.entity(hero).insert(player::Player::default());

    let mut inventory = player::inventory::create_inventory(10);
    let _ = inventory.add_item("hero".to_owned(), 1);
    let _ = inventory.add_item("terrain".to_owned(), 24);

    display_hud(
        &mut commands,
        asset_server.as_ref(),
        &create_global_item_catalog(),
        &hud::HudData {
            health: 100,
            energy: 50,
            inventory,
        },
    );
}

fn create_global_item_catalog() -> player::inventory::ItemCatalog {
    let mut item_catalog = player::inventory::ItemCatalog::default();

    item_catalog
}
