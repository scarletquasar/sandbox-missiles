use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    time::SystemTime,
};

use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    scenes::{
        effects::{JsonLevelTileEffect, TileEffects},
        SceneDefinition, SceneSprite, SceneTile,
    },
    sprites::pastoral_tileset::{PastoralSprite, PastoralTileSprite},
};

pub const LEVELS_DIRECTORY: &str = "assets/levels";
const DEFAULT_SCAN_INTERVAL_SECONDS: f32 = 0.5;

#[derive(Default)]
pub struct LevelReloadReport {
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Resource)]
pub struct LevelRegistry {
    directory: PathBuf,
    scan_timer: Timer,
    levels: HashMap<String, SceneDefinition>,
    modified_at: HashMap<String, SystemTime>,
}

impl Default for LevelRegistry {
    fn default() -> Self {
        Self {
            directory: PathBuf::from(LEVELS_DIRECTORY),
            scan_timer: Timer::from_seconds(DEFAULT_SCAN_INTERVAL_SECONDS, TimerMode::Repeating),
            levels: HashMap::new(),
            modified_at: HashMap::new(),
        }
    }
}

impl LevelRegistry {
    pub fn get(&self, name: &str) -> Option<&SceneDefinition> {
        self.levels.get(name)
    }

    pub fn first_level_name(&self) -> Option<&str> {
        self.levels.keys().min().map(String::as_str)
    }

    pub fn tick_scan_timer(&mut self, delta: std::time::Duration) -> bool {
        self.scan_timer.tick(delta).just_finished()
    }

    pub fn reload_from_disk(&mut self) -> LevelReloadReport {
        let mut report = LevelReloadReport::default();

        if let Err(error) = fs::create_dir_all(&self.directory) {
            report.errors.push(format!(
                "Unable to create level directory '{}': {error}",
                self.directory.display()
            ));
            return report;
        }

        let mut seen_levels = HashSet::new();
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) => {
                report.errors.push(format!(
                    "Unable to read level directory '{}': {error}",
                    self.directory.display()
                ));
                return report;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }

            let Some(level_name) = path
                .file_stem()
                .and_then(|file_stem| file_stem.to_str())
                .map(str::to_owned)
            else {
                continue;
            };

            seen_levels.insert(level_name.clone());

            let modified_at = match entry.metadata().and_then(|metadata| metadata.modified()) {
                Ok(modified_at) => modified_at,
                Err(error) => {
                    report.errors.push(format!(
                        "Unable to read metadata for level '{}': {error}",
                        path.display()
                    ));
                    continue;
                }
            };

            let should_reload = self
                .modified_at
                .get(&level_name)
                .map_or(true, |known_modified_at| *known_modified_at != modified_at);

            if !should_reload {
                continue;
            }

            match load_level_definition(&path) {
                Ok(definition) => {
                    self.levels.insert(level_name.clone(), definition);
                    self.modified_at.insert(level_name.clone(), modified_at);
                    report.changed.push(level_name);
                }
                Err(error) => report.errors.push(error),
            }
        }

        let removed_levels: Vec<String> = self
            .modified_at
            .keys()
            .filter(|level_name| !seen_levels.contains(*level_name))
            .cloned()
            .collect();

        for level_name in removed_levels {
            self.modified_at.remove(&level_name);
            self.levels.remove(&level_name);
            report.removed.push(level_name);
        }

        report
    }
}

fn load_level_definition(path: &PathBuf) -> Result<SceneDefinition, String> {
    let raw_level = fs::read_to_string(path)
        .map_err(|error| format!("Unable to read level '{}': {error}", path.display()))?;
    let parsed_level: JsonLevelDefinition = serde_json::from_str(&raw_level)
        .map_err(|error| format!("Invalid JSON in level '{}': {error}", path.display()))?;

    parsed_level
        .into_scene_definition()
        .map_err(|error| format!("Invalid level '{}': {error}", path.display()))
}

#[derive(Deserialize)]
struct JsonLevelDefinition {
    player_spawn: JsonGridPosition,
    tiles: Vec<Vec<JsonLevelTile>>,
}

impl JsonLevelDefinition {
    fn into_scene_definition(self) -> Result<SceneDefinition, String> {
        if self.tiles.is_empty() {
            return Err("tiles must contain at least one row".to_owned());
        }

        let row_width = self.tiles[0].len();
        if row_width == 0 {
            return Err("each level row must contain at least one tile".to_owned());
        }

        let mut tiles = Vec::with_capacity(self.tiles.len());
        for (row_index, row) in self.tiles.into_iter().enumerate() {
            if row.len() != row_width {
                return Err(format!(
                    "row {row_index} has width {}, expected {row_width}",
                    row.len()
                ));
            }

            let mut parsed_row = Vec::with_capacity(row_width);
            for tile in row {
                parsed_row.push(tile.into_scene_tile()?);
            }
            tiles.push(parsed_row);
        }

        Ok(SceneDefinition {
            tiles,
            player_spawn: self.player_spawn.as_ivec2(),
        })
    }
}

#[derive(Deserialize)]
struct JsonGridPosition {
    x: i32,
    y: i32,
}

impl JsonGridPosition {
    fn as_ivec2(&self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }
}

#[derive(Deserialize)]
struct JsonLevelTile {
    sprite: JsonSceneSprite,
    #[serde(default = "default_walkable")]
    walkable: bool,
    #[serde(default)]
    effects: Vec<JsonLevelTileEffect>,
    #[serde(default)]
    z_layer: f32,
    #[serde(default)]
    tag: Option<String>,
}

impl JsonLevelTile {
    fn into_scene_tile(self) -> Result<SceneTile, String> {
        Ok(SceneTile {
            sprite: self.sprite.into_scene_sprite()?,
            walkable: self.walkable,
            effects: TileEffects::from_json(self.effects)?,
            z_layer: self.z_layer,
            tag: self.tag,
        })
    }
}

#[derive(Deserialize)]
struct JsonSceneSprite {
    tileset: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    col: Option<u32>,
    #[serde(default)]
    row: Option<u32>,
}

impl JsonSceneSprite {
    fn into_scene_sprite(self) -> Result<SceneSprite, String> {
        match self.tileset.as_str() {
            "pastoral" => {
                if let Some(name) = self.name.as_deref() {
                    let sprite = PastoralSprite::from_name(name)
                        .ok_or_else(|| format!("unknown pastoral sprite name '{name}'"))?;
                    return Ok(SceneSprite::Pastoral(sprite.tile()));
                }

                if let (Some(col), Some(row)) = (self.col, self.row) {
                    return Ok(SceneSprite::Pastoral(PastoralTileSprite::from_grid(
                        col, row,
                    )));
                }

                Err("pastoral sprites require either 'name' or both 'col' and 'row'".to_owned())
            }
            other => Err(format!("unsupported tileset '{other}'")),
        }
    }
}

const fn default_walkable() -> bool {
    true
}
