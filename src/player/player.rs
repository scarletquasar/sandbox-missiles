use bevy::prelude::*;

use crate::{
    scenes::{CurrentScene, LevelRegistry, SceneDefinition, SceneTile},
    sprites::hero_tileset::{set_hero_animation, HeroAnimation, HeroDirection, HeroFacing},
    sprites::pastoral_tileset::TILE_WORLD_SIZE,
};

const PLAYER_SPEED: f32 = 120.0;
const PLAYER_MAX_HEALTH: f32 = 100.0;
const PLAYER_MAX_ENERGY: f32 = 100.0;
const PLAYER_ENERGY_REGEN_AMOUNT: f32 = 10.0;
const PLAYER_ENERGY_REGEN_INTERVAL_SECONDS: f32 = 5.0;
const PLAYER_COLLIDER_HALF_EXTENT: f32 = TILE_WORLD_SIZE * 0.5 - 0.001;
const COLLISION_STEP: f32 = 1.0;
const CAMERA_FOLLOW_SPEED_MULTIPLIER: f32 = 0.8;
const CAMERA_FOLLOW_SNAP_DISTANCE: f32 = 0.5;

#[derive(Component)]
pub struct Player {
    speed: f32,
    health: f32,
    energy: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: PLAYER_SPEED,
            health: PLAYER_MAX_HEALTH,
            energy: PLAYER_MAX_ENERGY,
        }
    }
}

impl Player {
    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn health_ratio(&self) -> f32 {
        player_stat_ratio(self.health, PLAYER_MAX_HEALTH)
    }

    pub fn energy_ratio(&self) -> f32 {
        player_stat_ratio(self.energy, PLAYER_MAX_ENERGY)
    }

    pub fn try_spend_energy(&mut self, amount: f32) -> bool {
        if self.energy < amount {
            return false;
        }

        self.energy -= amount;
        true
    }

    fn regenerate_energy(&mut self) {
        self.energy = (self.energy + PLAYER_ENERGY_REGEN_AMOUNT).min(PLAYER_MAX_ENERGY);
    }
}

fn player_stat_ratio(value: f32, max: f32) -> f32 {
    if max <= 0.0 {
        return 0.0;
    }

    (value / max).clamp(0.0, 1.0)
}

#[derive(Resource)]
pub struct PlayerEnergyRegenTimer(Timer);

impl Default for PlayerEnergyRegenTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            PLAYER_ENERGY_REGEN_INTERVAL_SECONDS,
            TimerMode::Repeating,
        ))
    }
}

pub fn regenerate_player_energy(
    time: Res<Time>,
    mut timer: ResMut<PlayerEnergyRegenTimer>,
    mut player_query: Query<&mut Player>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(mut player) = player_query.get_single_mut() else {
        return;
    };

    player.regenerate_energy();
}

pub fn move_player(
    current_scene: Res<CurrentScene>,
    keyboard: Res<ButtonInput<KeyCode>>,
    level_registry: Res<LevelRegistry>,
    time: Res<Time>,
    mut player_query: Query<(
        &mut Player,
        &mut Transform,
        &mut Sprite,
        &mut HeroAnimation,
        &mut HeroFacing,
    )>,
) {
    let Ok((mut player, mut transform, mut sprite, mut animation, mut facing)) =
        player_query.get_single_mut()
    else {
        return;
    };

    let scene = current_scene
        .current_name()
        .and_then(|scene_name| level_registry.get(scene_name));

    let (direction, hero_direction) = match keyboard.get_pressed().next().copied() {
        Some(KeyCode::KeyW) => (Vec2::new(0.0, 1.0), HeroDirection::Back),
        Some(KeyCode::KeyS) => (Vec2::new(0.0, -1.0), HeroDirection::Front),
        Some(KeyCode::KeyA) => (Vec2::new(-1.0, 0.0), HeroDirection::Left),
        Some(KeyCode::KeyD) => (Vec2::new(1.0, 0.0), HeroDirection::Right),
        _ => (Vec2::ZERO, facing.0),
    };

    let movement = player.speed * time.delta_secs();
    if direction != Vec2::ZERO && movement > 0.0 {
        let start_position = transform.translation;

        transform.translation.x = move_axis_x(scene, start_position, direction.x * movement);
        transform.translation.y = move_axis_y(scene, transform.translation, direction.y * movement);

        if let Some(scene) = scene {
            let tile = scene.tile_at(scene.world_to_grid(transform.translation));
            apply_tile_effects(tile, &mut player);
        }
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

pub fn follow_player_camera(
    time: Res<Time>,
    player_query: Query<(&Player, &Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok((player, player_transform)) = player_query.get_single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let current = camera_transform.translation.truncate();
    let target = player_transform.translation.truncate();
    let to_target = target - current;
    let distance = to_target.length();

    if distance <= CAMERA_FOLLOW_SNAP_DISTANCE {
        camera_transform.translation.x = target.x;
        camera_transform.translation.y = target.y;
        return;
    }

    let max_step = player.speed * CAMERA_FOLLOW_SPEED_MULTIPLIER * time.delta_secs();
    let next = current + to_target.normalize() * max_step.min(distance);

    camera_transform.translation.x = next.x;
    camera_transform.translation.y = next.y;
}

fn move_axis_x(scene: Option<&SceneDefinition>, current: Vec3, delta: f32) -> f32 {
    move_axis(scene, current, delta, Axis::X).x
}

fn move_axis_y(scene: Option<&SceneDefinition>, current: Vec3, delta: f32) -> f32 {
    move_axis(scene, current, delta, Axis::Y).y
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

fn move_axis(scene: Option<&SceneDefinition>, current: Vec3, delta: f32, axis: Axis) -> Vec3 {
    let Some(scene) = scene else {
        return current;
    };

    let mut remaining = delta.abs();
    let direction = delta.signum();
    let mut position = current;

    while remaining > 0.0 {
        let step = remaining.min(COLLISION_STEP) * direction;
        let mut candidate = position;

        match axis {
            Axis::X => candidate.x += step,
            Axis::Y => candidate.y += step,
        }

        if collides_with_blockage(scene, candidate) {
            break;
        }

        position = candidate;
        remaining -= step.abs();
    }

    position
}

fn collides_with_blockage(scene: &SceneDefinition, position: Vec3) -> bool {
    let min_x = position.x - PLAYER_COLLIDER_HALF_EXTENT;
    let max_x = position.x + PLAYER_COLLIDER_HALF_EXTENT;
    let min_y = position.y - PLAYER_COLLIDER_HALF_EXTENT;
    let max_y = position.y + PLAYER_COLLIDER_HALF_EXTENT;

    let top_left = scene.world_to_grid(Vec3::new(min_x, max_y, position.z));
    let bottom_right = scene.world_to_grid(Vec3::new(max_x, min_y, position.z));

    for y in top_left.y..=bottom_right.y {
        for x in top_left.x..=bottom_right.x {
            let Some(tile) = scene.tile_at(IVec2::new(x, y)) else {
                return true;
            };

            if tile
                .effects
                .effects
                .iter()
                .any(|effect| effect.tile_effect_type == "blockage")
            {
                return true;
            }
        }
    }

    false
}

fn apply_tile_effects(tile: Option<&SceneTile>, player: &mut Player) {
    let Some(tile) = tile else {
        return;
    };

    for effect in &tile.effects.effects {
        if effect.tile_effect_type == "damage" {
            player.health -= effect.modifier;
        } else if effect.tile_effect_type == "heal" {
            player.health += effect.modifier;
        } else if effect.tile_effect_type == "slow" {
        } else if effect.tile_effect_type == "speed" {
        } else if effect.tile_effect_type == "teleport" {
        } else if effect.tile_effect_type == "voyage" {
        } else if effect.tile_effect_type == "message" {
        } else if effect.tile_effect_type == "blockage" {
        }
    }
}
