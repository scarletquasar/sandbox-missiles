use bevy::prelude::*;

use crate::sprites::hero_tileset::{set_hero_animation, HeroAnimation, HeroDirection, HeroFacing};

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
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<(
        &Player,
        &mut Transform,
        &mut Sprite,
        &mut HeroAnimation,
        &mut HeroFacing,
    )>,
) {
    for (player, mut transform, mut sprite, mut animation, mut facing) in &mut players {
        let (direction, hero_direction) = match keyboard.get_pressed().next().copied() {
            Some(KeyCode::KeyW) => (Vec2::new(0.0, 1.0), HeroDirection::Back),
            Some(KeyCode::KeyS) => (Vec2::new(0.0, -1.0), HeroDirection::Front),
            Some(KeyCode::KeyA) => (Vec2::new(-1.0, 0.0), HeroDirection::Left),
            Some(KeyCode::KeyD) => (Vec2::new(1.0, 0.0), HeroDirection::Right),
            _ => (Vec2::ZERO, facing.0),
        };

        if direction == Vec2::ZERO {
            set_hero_animation(
                &mut sprite,
                &mut animation,
                &mut facing,
                hero_direction,
                false,
            );
            continue;
        }

        transform.translation.x += direction.x * player.speed * time.delta_secs();
        transform.translation.y += direction.y * player.speed * time.delta_secs();

        set_hero_animation(
            &mut sprite,
            &mut animation,
            &mut facing,
            hero_direction,
            true,
        );
    }
}
