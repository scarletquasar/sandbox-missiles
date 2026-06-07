pub mod effects;
pub mod level_registry;

use bevy::log::warn;
use bevy::prelude::*;

use crate::{
    player::Player,
    sprites::pastoral_tileset::{self, PastoralTileSprite, TILE_WORLD_SIZE},
};

pub use effects::TileEffects;
pub use level_registry::LevelRegistry;

#[derive(Clone)]
pub struct SpawnedScene {
    pub name: String,
    pub root: Entity,
    pub player_spawn: Vec3,
}

#[derive(Resource, Default, Clone)]
pub struct CurrentScene {
    scene: Option<SpawnedScene>,
}

impl CurrentScene {
    pub fn take(&mut self) -> Option<SpawnedScene> {
        self.scene.take()
    }

    pub fn replace(&mut self, scene: SpawnedScene) {
        self.scene = Some(scene);
    }

    pub fn current_name(&self) -> Option<&str> {
        self.scene.as_ref().map(|scene| scene.name.as_str())
    }
}

#[derive(Component)]
struct SceneRoot;

#[derive(Component, Clone)]
#[allow(dead_code)]
pub struct SpawnedSceneTile {
    pub grid_position: IVec2,
    pub walkable: bool,
}

#[derive(Clone)]
pub struct SceneTile {
    pub sprite: SceneSprite,
    pub walkable: bool,
    pub effects: TileEffects,
    pub z_layer: f32,
    pub tag: Option<String>,
}

#[derive(Clone, Copy)]
pub enum SceneSprite {
    Pastoral(PastoralTileSprite),
}

pub struct SceneDefinition {
    pub tiles: Vec<Vec<SceneTile>>,
    pub player_spawn: IVec2,
}

impl SceneDefinition {
    pub fn width(&self) -> i32 {
        self.tiles.first().map_or(0, |row| row.len() as i32)
    }

    pub fn height(&self) -> i32 {
        self.tiles.len() as i32
    }

    pub fn tile_at(&self, grid_position: IVec2) -> Option<&SceneTile> {
        if grid_position.x < 0 || grid_position.y < 0 {
            return None;
        }

        self.tiles
            .get(grid_position.y as usize)
            .and_then(|row| row.get(grid_position.x as usize))
    }

    pub fn grid_to_world(&self, grid_position: IVec2, z: f32) -> Vec3 {
        let world_width = self.width() as f32 * TILE_WORLD_SIZE;
        let world_height = self.height() as f32 * TILE_WORLD_SIZE;

        Vec3::new(
            -world_width * 0.5 + grid_position.x as f32 * TILE_WORLD_SIZE + TILE_WORLD_SIZE * 0.5,
            world_height * 0.5 - grid_position.y as f32 * TILE_WORLD_SIZE - TILE_WORLD_SIZE * 0.5,
            z - grid_position.y as f32 * 0.001,
        )
    }

    pub fn world_to_grid(&self, world_position: Vec3) -> IVec2 {
        let world_width = self.width() as f32 * TILE_WORLD_SIZE;
        let world_height = self.height() as f32 * TILE_WORLD_SIZE;
        let grid_x = ((world_position.x + world_width * 0.5) / TILE_WORLD_SIZE).floor() as i32;
        let grid_y = ((world_height * 0.5 - world_position.y) / TILE_WORLD_SIZE).floor() as i32;

        IVec2::new(grid_x, grid_y)
    }
}

pub fn spawn_scene(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    levels: &LevelRegistry,
    scene_name: &str,
    previous_scene: Option<SpawnedScene>,
) -> Result<SpawnedScene, String> {
    let scene = levels
        .get(scene_name)
        .ok_or_else(|| format!("Level '{scene_name}' not found in assets/levels"))?;

    if let Some(previous_scene) = previous_scene {
        commands.entity(previous_scene.root).despawn();
    }

    let player_spawn = scene.grid_to_world(scene.player_spawn, 0.75);
    let pastoral_atlas = pastoral_tileset::load_pastoral_atlas(asset_server, texture_atlas_layouts);
    let root = commands
        .spawn((
            Name::new(format!("{scene_name}Scene")),
            SceneRoot,
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        for (row_index, row) in scene.tiles.iter().enumerate() {
            for (col_index, tile) in row.iter().enumerate() {
                let grid_position = IVec2::new(col_index as i32, row_index as i32);
                let world_position = scene.grid_to_world(grid_position, tile.z_layer);

                match tile.sprite {
                    SceneSprite::Pastoral(sprite) => {
                        let mut entity = parent.spawn(pastoral_tileset::pastoral_tile_components(
                            &pastoral_atlas,
                            sprite,
                            world_position,
                        ));
                        entity.insert(SpawnedSceneTile {
                            grid_position,
                            walkable: tile.walkable,
                        });

                        if !tile.effects.is_empty() {
                            entity.insert(tile.effects.clone());
                        }

                        if let Some(tag) = &tile.tag {
                            entity.insert(Name::new(tag.clone()));
                        }
                    }
                }
            }
        }
    });

    Ok(SpawnedScene {
        name: scene_name.to_owned(),
        root,
        player_spawn,
    })
}

pub fn reload_levels_during_runtime(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_scene: ResMut<CurrentScene>,
    mut level_registry: ResMut<LevelRegistry>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !level_registry.tick_scan_timer(time.delta()) {
        return;
    }

    let report = level_registry.reload_from_disk();
    for error in &report.errors {
        warn!("{error}");
    }

    let Some(scene_name) = current_scene.current_name().map(str::to_owned) else {
        return;
    };

    if report
        .removed
        .iter()
        .any(|level_name| level_name == &scene_name)
    {
        warn!("Current level '{scene_name}' was removed from assets/levels; keeping the spawned scene");
        return;
    }

    if !report
        .changed
        .iter()
        .any(|level_name| level_name == &scene_name)
    {
        return;
    }

    let fallback_scene = current_scene.take();
    let previous_scene = fallback_scene.clone();

    match spawn_scene(
        &mut commands,
        asset_server.as_ref(),
        texture_atlas_layouts.as_mut(),
        level_registry.as_ref(),
        &scene_name,
        previous_scene,
    ) {
        Ok(scene) => {
            if let Ok(mut transform) = player_query.get_single_mut() {
                transform.translation = scene.player_spawn;
            }

            current_scene.replace(scene);
        }
        Err(error) => {
            warn!("{error}");

            if let Some(fallback_scene) = fallback_scene {
                current_scene.replace(fallback_scene);
            }
        }
    }
}
