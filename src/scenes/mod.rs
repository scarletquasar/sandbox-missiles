pub mod main_scene;

use bevy::prelude::*;

use crate::sprites::pastoral_tileset::{self, PastoralTileSprite, TILE_WORLD_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneName {
    Main,
}

#[derive(Clone, Copy)]
pub struct SpawnedScene {
    pub name: SceneName,
    pub root: Entity,
    pub player_spawn: Vec3,
}

#[derive(Component)]
struct SceneRoot {
    name: SceneName,
}

#[derive(Clone)]
pub struct SceneTile {
    pub sprite: SceneSprite,
    pub walkable: bool,
    pub z_layer: f32,
    pub tag: Option<&'static str>,
}

impl SceneTile {
    pub fn new(sprite: SceneSprite) -> Self {
        Self {
            sprite,
            walkable: true,
            z_layer: 0.0,
            tag: None,
        }
    }

    pub fn with_walkable(mut self, walkable: bool) -> Self {
        self.walkable = walkable;
        self
    }

    pub fn with_z_layer(mut self, z_layer: f32) -> Self {
        self.z_layer = z_layer;
        self
    }

    pub fn with_tag(mut self, tag: &'static str) -> Self {
        self.tag = Some(tag);
        self
    }
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
    pub fn filled(width: usize, height: usize, tile: SceneTile, player_spawn: IVec2) -> Self {
        Self {
            tiles: vec![vec![tile; width]; height],
            player_spawn,
        }
    }

    pub fn width(&self) -> i32 {
        self.tiles.first().map_or(0, |row| row.len() as i32)
    }

    pub fn height(&self) -> i32 {
        self.tiles.len() as i32
    }

    pub fn set_tile(&mut self, grid_position: IVec2, tile: SceneTile) {
        if grid_position.x < 0
            || grid_position.y < 0
            || grid_position.x >= self.width()
            || grid_position.y >= self.height()
        {
            return;
        }

        self.tiles[grid_position.y as usize][grid_position.x as usize] = tile;
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
}

pub fn spawn_scene(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    scene_name: SceneName,
    previous_scene: Option<SpawnedScene>,
) -> SpawnedScene {
    if let Some(previous_scene) = previous_scene {
        commands.entity(previous_scene.root).despawn();
    }

    let scene = scene_definition(scene_name);
    let player_spawn = scene.grid_to_world(scene.player_spawn, 0.75);
    let pastoral_atlas = pastoral_tileset::load_pastoral_atlas(asset_server, texture_atlas_layouts);
    let root = commands
        .spawn((
            Name::new(format!("{scene_name:?}Scene")),
            SceneRoot { name: scene_name },
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

                        if let Some(tag) = tile.tag {
                            entity.insert(Name::new(tag));
                        }
                    }
                }
            }
        }
    });

    SpawnedScene {
        name: scene_name,
        root,
        player_spawn,
    }
}

fn scene_definition(scene_name: SceneName) -> SceneDefinition {
    match scene_name {
        SceneName::Main => main_scene::build_main_scene(),
    }
}
