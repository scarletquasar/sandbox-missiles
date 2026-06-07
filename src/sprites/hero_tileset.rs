use std::time::Duration;

use bevy::prelude::*;

pub const HERO_PATH: &str = "textures/hero.png";

const HERO_FRAME_SIZE: UVec2 = UVec2::new(16, 16);
const HERO_COLUMNS: u32 = 8;
const HERO_ROWS: u32 = 3;
const HERO_SCALE: f32 = 2.0;
const HERO_ANIMATION_FPS: u8 = 10;

#[derive(Component)]
pub struct Hero;

#[derive(Component, Debug, Clone, Copy)]
pub struct HeroFacing(pub HeroDirection);

#[derive(Debug, Clone, Copy)]
pub enum HeroDirection {
    Back,
    Front,
    Left,
    Right,
}

#[derive(Component)]
pub struct HeroAnimation {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

impl HeroAnimation {
    fn idle(direction: HeroDirection) -> Self {
        let idle_index = idle_frame(direction);
        Self {
            first_sprite_index: idle_index,
            last_sprite_index: idle_index,
            fps: HERO_ANIMATION_FPS,
            frame_timer: Self::timer_from_fps(HERO_ANIMATION_FPS),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / fps as f32), TimerMode::Once)
    }
}

#[derive(Clone)]
struct HeroAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

pub fn spawn_hero(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    world_position: Vec3,
    direction: HeroDirection,
) -> Entity {
    let atlas = load_hero_atlas(asset_server, texture_atlas_layouts);
    let animation = HeroAnimation::idle(direction);

    commands
        .spawn((
            Name::new("Hero"),
            Hero,
            HeroFacing(direction),
            Sprite::from_atlas_image(
                atlas.image,
                TextureAtlas {
                    layout: atlas.layout,
                    index: animation.first_sprite_index,
                },
            ),
            Transform::from_translation(world_position).with_scale(Vec3::splat(HERO_SCALE)),
            animation,
        ))
        .id()
}

pub fn set_hero_animation(
    sprite: &mut Sprite,
    animation: &mut HeroAnimation,
    facing: &mut HeroFacing,
    direction: HeroDirection,
    is_moving: bool,
) {
    facing.0 = direction;
    let (first_sprite_index, last_sprite_index) = if is_moving {
        walking_frames(direction)
    } else {
        let idle_index = idle_frame(direction);
        (idle_index, idle_index)
    };

    let changed = animation.first_sprite_index != first_sprite_index
        || animation.last_sprite_index != last_sprite_index;

    if changed {
        animation.first_sprite_index = first_sprite_index;
        animation.last_sprite_index = last_sprite_index;
        animation.frame_timer = HeroAnimation::timer_from_fps(animation.fps);

        if let Some(texture_atlas) = &mut sprite.texture_atlas {
            texture_atlas.index = first_sprite_index;
        }
    }
}

pub fn execute_hero_animations(
    time: Res<Time>,
    mut query: Query<(&mut HeroAnimation, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in &mut query {
        if animation.first_sprite_index == animation.last_sprite_index {
            continue;
        }

        animation.frame_timer.tick(time.delta());
        if animation.frame_timer.just_finished() {
            if let Some(texture_atlas) = &mut sprite.texture_atlas {
                if texture_atlas.index >= animation.last_sprite_index
                    || texture_atlas.index < animation.first_sprite_index
                {
                    texture_atlas.index = animation.first_sprite_index;
                } else {
                    texture_atlas.index += 1;
                }
            }

            animation.frame_timer = HeroAnimation::timer_from_fps(animation.fps);
        }
    }
}

fn walking_frames(direction: HeroDirection) -> (usize, usize) {
    match direction {
        HeroDirection::Back => (0, 3),
        HeroDirection::Front => (4, 7),
        HeroDirection::Left => (8, 11),
        HeroDirection::Right => (12, 15),
    }
}

fn idle_frame(direction: HeroDirection) -> usize {
    match direction {
        HeroDirection::Back => 1,
        HeroDirection::Front => 6,
        HeroDirection::Left => 9,
        HeroDirection::Right => 13,
    }
}

fn load_hero_atlas(
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> HeroAtlas {
    let image = asset_server.load(HERO_PATH);
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        HERO_FRAME_SIZE,
        HERO_COLUMNS,
        HERO_ROWS,
        None,
        None,
    ));

    HeroAtlas { image, layout }
}
