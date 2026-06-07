use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Clone, Debug, Default)]
pub struct TileEffects {
    pub effects: Vec<TileEffect>,
}

impl TileEffects {
    pub(in crate::scenes) fn from_json(effects: Vec<JsonLevelTileEffect>) -> Result<Self, String> {
        effects
            .into_iter()
            .map(JsonLevelTileEffect::into_tile_effect)
            .collect::<Result<Vec<_>, _>>()
            .map(|effects| Self { effects })
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TileEffect {
    // Types:
    // - "damage": applies damage to the player when stepped on, modifier is the amount of damage
    // - "heal": heals the player when stepped on, modifier is the amount of healing
    // - "slow": slows the player when stepped on, modifier is the percentage to slow
    // - "speed": speeds up the player when stepped on, modifier is the percentage to speed up
    // - "teleport": teleports the player to a different location, modifier is ignored, extra_data is the destination in the format "x,y"
    // - "voyage": mover the player to a different scene, modifier is fade duration, extra_data is the name of the scene to move to
    // - "message": shows a message to the player, modifier is ignored, extra_data is the message to show
    // - "blockage": the player cannot step on this tile, modifier is ignored, extra_data is ignored
    pub tile_effect_type: String,
    pub modifier: f32,
    pub extra_data: Option<String>,
}

#[derive(Deserialize)]
pub(in crate::scenes) struct JsonLevelTileEffect {
    tile_effect_type: String,
    modifier: f32,
    extra_data: Option<String>,
}

impl JsonLevelTileEffect {
    fn into_tile_effect(self) -> Result<TileEffect, String> {
        let tile_effect_type = self.tile_effect_type.trim();
        if tile_effect_type.is_empty() {
            return Err("tile effect type must not be empty".to_owned());
        }

        if !self.modifier.is_finite() {
            return Err(format!(
                "tile effect '{tile_effect_type}' modifier must be finite"
            ));
        }

        let extra_data = self
            .extra_data
            .map(|extra_data| extra_data.trim().to_owned())
            .filter(|extra_data| !extra_data.is_empty());

        Ok(TileEffect {
            tile_effect_type: tile_effect_type.to_owned(),
            modifier: self.modifier,
            extra_data,
        })
    }
}
