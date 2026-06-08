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
        .init_resource::<player::PlayerEnergyRegenTimer>()
        .init_resource::<player::SelectedInventorySlot>()
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
                (player::move_player, player::follow_player_camera).chain(),
                (
                    player::select_inventory_item,
                    player::drop_selected_item_effect,
                    player::regenerate_player_energy,
                    hud::update_player_hud,
                    hud::update_inventory_hud,
                )
                    .chain(),
                player::animate_potion_missiles,
                player::animate_potion_impacts,
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
    mut item_catalog: ResMut<player::inventory::ItemCatalog>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Instances camera, hud and player
    let camera = commands.spawn(Camera2d).id();
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
    commands
        .entity(camera)
        .insert(Transform::from_xyz(player_spawn.x, player_spawn.y, 0.0));

    let hero = sprites::hero_tileset::spawn_hero(
        &mut commands,
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
        player_spawn,
        sprites::hero_tileset::HeroDirection::Front,
    );
    commands.entity(hero).insert(player::Player::default());

    *item_catalog = create_global_item_catalog();

    let mut inventory = player::inventory::create_inventory(10);
    inventory
        .add_item(item_catalog.as_ref(), "potion_red".to_owned(), 1)
        .expect("Item 'potion_red' must exist in the global item catalog");
    inventory
        .add_item(item_catalog.as_ref(), "potion_green".to_owned(), 1)
        .expect("Item 'potion_green' must exist in the global item catalog");
    inventory
        .add_item(item_catalog.as_ref(), "potion_blue".to_owned(), 1)
        .expect("Item 'potion_blue' must exist in the global item catalog");
    inventory
        .add_item(item_catalog.as_ref(), "potion_black".to_owned(), 1)
        .expect("Item 'potion_black' must exist in the global item catalog");

    display_hud(
        &mut commands,
        asset_server.as_ref(),
        &inventory,
        item_catalog.as_ref(),
        &hud::HudData {
            health: 100,
            energy: 100,
        },
    );

    commands.insert_resource(inventory);
}

fn create_global_item_catalog() -> player::inventory::ItemCatalog {
    let mut item_catalog = player::inventory::ItemCatalog::default();
    item_catalog.effects.insert(
        "red_missile".to_owned(),
        player::inventory::Effect::DamageToEnemies(20.0, 2.0),
    );
    item_catalog.effects.insert(
        "green_missile".to_owned(),
        player::inventory::Effect::DamageToEnemies(30.0, 4.0),
    );
    item_catalog.effects.insert(
        "blue_missile".to_owned(),
        player::inventory::Effect::DamageToEnemies(40.0, 6.0),
    );
    item_catalog.effects.insert(
        "black_missile".to_owned(),
        player::inventory::Effect::DamageToEnemies(55.0, 9.0),
    );

    let potion_red = player::inventory::create_item(
        "potion_red".to_owned(),
        "Red Potion".to_owned(),
        "Calls a small crimson missile".to_owned(),
        "red_missile".to_owned(),
        "textures/potion_red.png".to_owned(),
    );
    item_catalog.items.insert(potion_red.id.clone(), potion_red);

    let potion_green = player::inventory::create_item(
        "potion_green".to_owned(),
        "Green Potion".to_owned(),
        "Calls a wider verdant missile".to_owned(),
        "green_missile".to_owned(),
        "textures/potion_green.png".to_owned(),
    );
    item_catalog
        .items
        .insert(potion_green.id.clone(), potion_green);

    let potion_blue = player::inventory::create_item(
        "potion_blue".to_owned(),
        "Blue Potion".to_owned(),
        "Calls a broad azure missile".to_owned(),
        "blue_missile".to_owned(),
        "textures/potion_blue.png".to_owned(),
    );
    item_catalog
        .items
        .insert(potion_blue.id.clone(), potion_blue);

    let potion_black = player::inventory::create_item(
        "potion_black".to_owned(),
        "Black Potion".to_owned(),
        "Calls the largest shadow missile".to_owned(),
        "black_missile".to_owned(),
        "textures/potion_black.png".to_owned(),
    );
    item_catalog
        .items
        .insert(potion_black.id.clone(), potion_black);

    item_catalog
}
