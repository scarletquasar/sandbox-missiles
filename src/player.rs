use bevy::prelude::*;

use crate::{
    scenes::{CurrentScene, LevelRegistry, SceneDefinition, SceneTile},
    sprites::hero_tileset::{set_hero_animation, HeroAnimation, HeroDirection, HeroFacing},
};

const PLAYER_SPEED: f32 = 120.0;

#[derive(Component)]
pub struct Player {
    speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: PLAYER_SPEED,
        }
    }
}

pub fn move_player(
    current_scene: Res<CurrentScene>,
    keyboard: Res<ButtonInput<KeyCode>>,
    level_registry: Res<LevelRegistry>,
    time: Res<Time>,
    mut player_query: Query<(
        &Player,
        &mut Transform,
        &mut Sprite,
        &mut HeroAnimation,
        &mut HeroFacing,
    )>,
) {
    let scene = current_scene
        .current_name()
        .and_then(|scene_name| level_registry.get(scene_name));

    let Ok((player, mut transform, mut sprite, mut animation, mut facing)) =
        player_query.get_single_mut()
    else {
        return;
    };

    let (direction, hero_direction) = match keyboard.get_pressed().next().copied() {
        Some(KeyCode::KeyW) => (Vec2::new(0.0, 1.0), HeroDirection::Back),
        Some(KeyCode::KeyS) => (Vec2::new(0.0, -1.0), HeroDirection::Front),
        Some(KeyCode::KeyA) => (Vec2::new(-1.0, 0.0), HeroDirection::Left),
        Some(KeyCode::KeyD) => (Vec2::new(1.0, 0.0), HeroDirection::Right),
        _ => (Vec2::ZERO, facing.0),
    };

    let next_tiles = adjacent_tiles(scene, transform.translation);
    let target_tile = if direction.y > 0.0 {
        next_tiles.up
    } else if direction.y < 0.0 {
        next_tiles.down
    } else if direction.x < 0.0 {
        next_tiles.left
    } else if direction.x > 0.0 {
        next_tiles.right
    } else {
        None
    };

    let can_move = apply_tile_effects(
        target_tile,
        player,
        &mut transform,
        &mut sprite,
        &mut animation,
        &mut facing,
    );

    if direction != Vec2::ZERO && can_move {
        transform.translation.x += direction.x * player.speed * time.delta_secs();
        transform.translation.y += direction.y * player.speed * time.delta_secs();
    }

    transform.translation.x = transform.translation.x.round();
    transform.translation.y = transform.translation.y.round();

    set_hero_animation(
        &mut sprite,
        &mut animation,
        &mut facing,
        hero_direction,
        direction != Vec2::ZERO,
    );
}

#[derive(Default)]
struct AdjacentTiles<'a> {
    up: Option<&'a SceneTile>,
    down: Option<&'a SceneTile>,
    left: Option<&'a SceneTile>,
    right: Option<&'a SceneTile>,
}

fn adjacent_tiles(scene: Option<&SceneDefinition>, player_position: Vec3) -> AdjacentTiles<'_> {
    let Some(scene) = scene else {
        return AdjacentTiles::default();
    };

    let player_grid_position = scene.world_to_grid(player_position);

    AdjacentTiles {
        up: scene.tile_at(player_grid_position + IVec2::new(0, -1)),
        down: scene.tile_at(player_grid_position + IVec2::new(0, 1)),
        left: scene.tile_at(player_grid_position + IVec2::new(-1, 0)),
        right: scene.tile_at(player_grid_position + IVec2::new(1, 0)),
    }
}

fn apply_tile_effects(
    tile: Option<&SceneTile>,
    _player: &Player,
    _transform: &mut Transform,
    _sprite: &mut Sprite,
    _animation: &mut HeroAnimation,
    _facing: &mut HeroFacing,
) -> bool {
    let Some(tile) = tile else {
        return true;
    };

    for effect in &tile.effects.effects {
        if effect.tile_effect_type == "damage" {
        } else if effect.tile_effect_type == "heal" {
        } else if effect.tile_effect_type == "slow" {
        } else if effect.tile_effect_type == "speed" {
        } else if effect.tile_effect_type == "teleport" {
        } else if effect.tile_effect_type == "voyage" {
        } else if effect.tile_effect_type == "message" {
        } else if effect.tile_effect_type == "blockage" {
            return false;
        }
    }

    true
}
