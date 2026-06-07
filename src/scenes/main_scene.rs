use bevy::prelude::*;

use crate::{
    scenes::{SceneDefinition, SceneSprite, SceneTile},
    sprites::pastoral_tileset::{region_tiles, PastoralRegion, PastoralSprite},
};

const SCENE_WIDTH: usize = 28;
const SCENE_HEIGHT: usize = 18;

pub fn build_main_scene() -> SceneDefinition {
    let mut scene = SceneDefinition::filled(
        SCENE_WIDTH,
        SCENE_HEIGHT,
        SceneTile::new(SceneSprite::Pastoral(PastoralSprite::GrassPlain.tile())),
        IVec2::new(12, 9),
    );

    for position in [
        IVec2::new(12, 1),
        IVec2::new(13, 1),
        IVec2::new(21, 2),
        IVec2::new(22, 1),
        IVec2::new(23, 2),
    ] {
        stamp_sprite(
            &mut scene,
            position,
            PastoralSprite::GrassTuftLeft,
            0.1,
            true,
            "grass_tuft",
        );
    }

    for position in [IVec2::new(14, 2), IVec2::new(24, 2)] {
        stamp_sprite(
            &mut scene,
            position,
            PastoralSprite::GrassTuftRight,
            0.1,
            true,
            "grass_tuft",
        );
    }

    for position in [IVec2::new(12, 2), IVec2::new(22, 3)] {
        stamp_sprite(
            &mut scene,
            position,
            PastoralSprite::GrassTuftLower,
            0.1,
            true,
            "grass_tuft",
        );
    }

    stamp_region(
        &mut scene,
        IVec2::new(1, 1),
        PastoralRegion::Pond,
        0.2,
        false,
        "pond",
    );
    stamp_sprite(
        &mut scene,
        IVec2::new(7, 3),
        PastoralSprite::OpenWater,
        0.2,
        false,
        "water",
    );
    stamp_sprite(
        &mut scene,
        IVec2::new(8, 4),
        PastoralSprite::OpenWater,
        0.2,
        false,
        "water",
    );

    stamp_region(
        &mut scene,
        IVec2::new(9, 1),
        PastoralRegion::Sign,
        0.3,
        false,
        "sign",
    );
    stamp_region(
        &mut scene,
        IVec2::new(24, 1),
        PastoralRegion::Parchment,
        0.3,
        false,
        "parchment",
    );

    stamp_region(
        &mut scene,
        IVec2::new(9, 6),
        PastoralRegion::TrailClearing,
        0.3,
        true,
        "trail",
    );
    stamp_region(
        &mut scene,
        IVec2::new(7, 6),
        PastoralRegion::PathVertical,
        0.3,
        true,
        "trail",
    );
    stamp_region(
        &mut scene,
        IVec2::new(7, 10),
        PastoralRegion::CurvedTrail,
        0.3,
        true,
        "trail",
    );

    stamp_region(
        &mut scene,
        IVec2::new(2, 7),
        PastoralRegion::Tree,
        0.4,
        false,
        "tree",
    );
    stamp_region(
        &mut scene,
        IVec2::new(2, 10),
        PastoralRegion::FruitTree,
        0.4,
        false,
        "tree",
    );
    stamp_sprite(
        &mut scene,
        IVec2::new(1, 12),
        PastoralSprite::TreeStump,
        0.4,
        false,
        "stump",
    );

    stamp_region(
        &mut scene,
        IVec2::new(5, 8),
        PastoralRegion::CottageWhite,
        0.5,
        false,
        "house",
    );
    stamp_region(
        &mut scene,
        IVec2::new(5, 11),
        PastoralRegion::CottageGray,
        0.5,
        false,
        "house",
    );

    stamp_region(
        &mut scene,
        IVec2::new(0, 14),
        PastoralRegion::Field,
        0.3,
        true,
        "field",
    );
    stamp_region(
        &mut scene,
        IVec2::new(8, 15),
        PastoralRegion::Flowers,
        0.4,
        true,
        "flowers",
    );
    stamp_region(
        &mut scene,
        IVec2::new(12, 15),
        PastoralRegion::Campfire,
        0.4,
        false,
        "campfire",
    );

    stamp_region(
        &mut scene,
        IVec2::new(18, 10),
        PastoralRegion::BigMountain,
        0.6,
        false,
        "mountain",
    );
    stamp_region(
        &mut scene,
        IVec2::new(23, 10),
        PastoralRegion::SmallMountainTop,
        0.6,
        false,
        "mountain",
    );
    stamp_region(
        &mut scene,
        IVec2::new(23, 13),
        PastoralRegion::SmallMountainBottom,
        0.6,
        false,
        "mountain",
    );

    scene
}

fn stamp_sprite(
    scene: &mut SceneDefinition,
    grid_position: IVec2,
    sprite: PastoralSprite,
    z_layer: f32,
    walkable: bool,
    tag: &'static str,
) {
    scene.set_tile(
        grid_position,
        SceneTile::new(SceneSprite::Pastoral(sprite.tile()))
            .with_z_layer(z_layer)
            .with_walkable(walkable)
            .with_tag(tag),
    );
}

fn stamp_region(
    scene: &mut SceneDefinition,
    origin: IVec2,
    region: PastoralRegion,
    z_layer: f32,
    walkable: bool,
    tag: &'static str,
) {
    for (offset, sprite) in region_tiles(region) {
        scene.set_tile(
            origin + offset,
            SceneTile::new(SceneSprite::Pastoral(sprite))
                .with_z_layer(z_layer)
                .with_walkable(walkable)
                .with_tag(tag),
        );
    }
}
