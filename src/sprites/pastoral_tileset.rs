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

const fn tile(col: u32, row: u32) -> PastoralTileSprite {
    PastoralTileSprite { col, row }
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
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "grass_plain" => Some(Self::GrassPlain),
            "grass_tuft_left" => Some(Self::GrassTuftLeft),
            "grass_tuft_right" => Some(Self::GrassTuftRight),
            "grass_tuft_lower" => Some(Self::GrassTuftLower),
            "open_water" => Some(Self::OpenWater),
            "tree_stump" => Some(Self::TreeStump),
            // fallback to open water so we will not have the game panicking when loading
            // invalid tiles on maps
            _ => Some(Self::OpenWater),
        }
    }

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

impl PastoralTileSprite {
    pub const fn from_grid(col: u32, row: u32) -> Self {
        Self { col, row }
    }

    fn atlas_index(self) -> usize {
        (self.row * TILE_COLUMNS + self.col) as usize
    }
}
