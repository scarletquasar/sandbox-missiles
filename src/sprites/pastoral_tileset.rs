use bevy::prelude::*;

pub const TILESET_PATH: &str = "pastoral-tileset.png";

const TILE_SIZE: UVec2 = UVec2::new(16, 16);
const TILE_SCALE: f32 = 2.0;
const TILE_COLUMNS: u32 = 12;
const TILE_ROWS: u32 = 16;

pub const TILE_WORLD_SIZE: f32 = TILE_SIZE.x as f32 * TILE_SCALE;

#[derive(Clone)]
pub struct PastoralAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PastoralTileSprite {
    col: u32,
    row: u32,
}

#[derive(Debug, Clone, Copy)]
struct AtlasRegion {
    min: PastoralTileSprite,
    width: u32,
    height: u32,
}

const fn tile(col: u32, row: u32) -> PastoralTileSprite {
    PastoralTileSprite { col, row }
}

const fn region(col: u32, row: u32, width: u32, height: u32) -> AtlasRegion {
    AtlasRegion {
        min: tile(col, row),
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PastoralSprite {
    GrassPlain,
    GrassTuftLeft,
    GrassTuftRight,
    GrassTuftLower,
    OpenWater,
    TreeStump,
}

impl PastoralSprite {
    pub const fn tile(self) -> PastoralTileSprite {
        match self {
            Self::GrassPlain => tile(6, 0),
            Self::GrassTuftLeft => tile(8, 0),
            Self::GrassTuftRight => tile(9, 0),
            Self::GrassTuftLower => tile(9, 1),
            Self::OpenWater => tile(6, 2),
            Self::TreeStump => tile(0, 10),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PastoralRegion {
    Pond,
    Sign,
    Parchment,
    Tree,
    FruitTree,
    CottageWhite,
    CottageGray,
    PathVertical,
    TrailClearing,
    CurvedTrail,
    Field,
    Flowers,
    Campfire,
    BigMountain,
    SmallMountainTop,
    SmallMountainBottom,
}

impl PastoralRegion {
    const fn atlas_region(self) -> AtlasRegion {
        match self {
            Self::Pond => region(0, 0, 6, 6),
            Self::Sign => region(10, 0, 2, 2),
            Self::Parchment => region(10, 2, 2, 4),
            Self::Tree => region(0, 6, 2, 2),
            Self::FruitTree => region(0, 8, 2, 2),
            Self::CottageWhite => region(2, 6, 2, 2),
            Self::CottageGray => region(2, 8, 2, 2),
            Self::PathVertical => region(4, 6, 2, 4),
            Self::TrailClearing => region(6, 6, 6, 6),
            Self::CurvedTrail => region(2, 10, 4, 2),
            Self::Field => region(0, 12, 6, 2),
            Self::Flowers => region(0, 14, 2, 2),
            Self::Campfire => region(2, 14, 4, 2),
            Self::BigMountain => region(6, 12, 4, 4),
            Self::SmallMountainTop => region(10, 12, 2, 2),
            Self::SmallMountainBottom => region(10, 14, 2, 2),
        }
    }
}

pub fn load_pastoral_atlas(
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> PastoralAtlas {
    let image = asset_server.load(TILESET_PATH);
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        TILE_SIZE,
        TILE_COLUMNS,
        TILE_ROWS,
        None,
        None,
    ));

    PastoralAtlas { image, layout }
}

pub fn spawn_pastoral_tile(
    commands: &mut Commands,
    atlas: &PastoralAtlas,
    sprite: PastoralTileSprite,
    world_position: Vec3,
) {
    commands.spawn(pastoral_tile_components(atlas, sprite, world_position));
}

pub fn pastoral_tile_components(
    atlas: &PastoralAtlas,
    sprite: PastoralTileSprite,
    world_position: Vec3,
) -> (Sprite, Transform) {
    (
        Sprite::from_atlas_image(
            atlas.image.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: sprite.atlas_index(),
            },
        ),
        Transform::from_translation(world_position).with_scale(Vec3::splat(TILE_SCALE)),
    )
}

pub fn region_tiles(region: PastoralRegion) -> Vec<(IVec2, PastoralTileSprite)> {
    let region = region.atlas_region();
    let mut tiles = Vec::with_capacity((region.width * region.height) as usize);

    for row in 0..region.height {
        for col in 0..region.width {
            tiles.push((
                IVec2::new(col as i32, row as i32),
                tile(region.min.col + col, region.min.row + row),
            ));
        }
    }

    tiles
}

impl PastoralTileSprite {
    fn atlas_index(self) -> usize {
        (self.row * TILE_COLUMNS + self.col) as usize
    }
}
