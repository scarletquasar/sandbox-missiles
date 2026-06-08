use bevy::prelude::*;

use crate::player::{
    inventory::{Inventory, ItemCatalog},
    Player, SelectedInventorySlot,
};

pub struct HudData {
    pub health: u32,
    pub energy: u32,
}

const INVENTORY_SLOT_SIZE: f32 = 56.0;
const INVENTORY_SLOT_GAP: f32 = 6.0;
const HEALTH_BAR_TEXTURE: &str = "textures/bar_red.png";
const ENERGY_BAR_TEXTURE: &str = "textures/bar_green.png";
const EMPTY_BAR_TEXTURE: &str = "textures/bar_empty.png";
const PLAYER_BAR_WIDTH: f32 = 206.0;
const PLAYER_BAR_HEIGHT: f32 = 27.0;
const PLAYER_BAR_LEFT: f32 = 12.0;
const PLAYER_BAR_TOP: f32 = 8.0;
const PLAYER_BAR_GAP: f32 = 2.0;
const PLAYER_BAR_MAX_VALUE: f32 = 100.0;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct EnergyBarFill;

#[derive(Component)]
pub struct InventoryHudRoot;

pub fn display_hud(
    commands: &mut Commands,
    asset_server: &AssetServer,
    inventory: &Inventory,
    item_catalog: &ItemCatalog,
    hud_data: &HudData,
) {
    let empty_bar_texture = asset_server.load(EMPTY_BAR_TEXTURE);
    let health_bar_texture = asset_server.load(HEALTH_BAR_TEXTURE);
    let energy_bar_texture = asset_server.load(ENERGY_BAR_TEXTURE);

    spawn_player_bar(
        commands,
        empty_bar_texture.clone(),
        health_bar_texture,
        PLAYER_BAR_TOP,
        hud_value_ratio(hud_data.health),
        HealthBarFill,
    );
    spawn_player_bar(
        commands,
        empty_bar_texture,
        energy_bar_texture,
        PLAYER_BAR_TOP + PLAYER_BAR_HEIGHT + PLAYER_BAR_GAP,
        hud_value_ratio(hud_data.energy),
        EnergyBarFill,
    );

    spawn_inventory_hud(commands, asset_server, inventory, item_catalog, None);
}

pub fn update_player_hud(
    player_query: Query<&Player>,
    mut health_bar_fills: Query<&mut Node, (With<HealthBarFill>, Without<EnergyBarFill>)>,
    mut energy_bar_fills: Query<&mut Node, (With<EnergyBarFill>, Without<HealthBarFill>)>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    for mut node in &mut health_bar_fills {
        set_bar_fill_ratio(&mut node, player.health_ratio());
    }

    for mut node in &mut energy_bar_fills {
        set_bar_fill_ratio(&mut node, player.energy_ratio());
    }
}

pub fn update_inventory_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inventory: Res<Inventory>,
    item_catalog: Res<ItemCatalog>,
    selected_slot: Res<SelectedInventorySlot>,
    inventory_roots: Query<Entity, With<InventoryHudRoot>>,
) {
    if !inventory.is_changed() && !selected_slot.is_changed() {
        return;
    }

    for entity in &inventory_roots {
        commands.entity(entity).despawn_recursive();
    }

    spawn_inventory_hud(
        &mut commands,
        asset_server.as_ref(),
        inventory.as_ref(),
        item_catalog.as_ref(),
        selected_slot.slot_index,
    );
}

fn spawn_inventory_hud(
    commands: &mut Commands,
    asset_server: &AssetServer,
    inventory: &Inventory,
    item_catalog: &ItemCatalog,
    selected_slot: Option<usize>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(
                        -((INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP)
                            * inventory.max_inventory_size as f32
                            * 0.5),
                    ),
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(INVENTORY_SLOT_GAP),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.96, 0.92, 0.78, 0.94)),
            BorderColor(Color::srgb(0.20, 0.16, 0.10)),
            InventoryHudRoot,
        ))
        .with_children(|parent| {
            for slot_index in 0..inventory.max_inventory_size {
                let item = inventory.items.get(slot_index);
                let is_selected = selected_slot == Some(slot_index);
                let slot_background = if is_selected {
                    Color::srgba(0.72, 0.84, 1.0, 0.98)
                } else {
                    Color::srgba(0.98, 0.97, 0.92, 0.96)
                };
                let slot_border = if is_selected {
                    Color::srgb(0.16, 0.46, 1.0)
                } else {
                    Color::srgb(0.28, 0.24, 0.18)
                };

                parent
                    .spawn((
                        Node {
                            width: Val::Px(INVENTORY_SLOT_SIZE),
                            height: Val::Px(INVENTORY_SLOT_SIZE),
                            border: UiRect::all(Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BackgroundColor(slot_background),
                        BorderColor(slot_border),
                    ))
                    .with_children(|slot| {
                        if let Some(item) = item {
                            if let Some(item_definition) = item_catalog.items.get(&item.item_id) {
                                slot.spawn((
                                    ImageNode::new(
                                        asset_server.load(item_definition.sprite_path.clone()),
                                    ),
                                    Node {
                                        width: Val::Px(INVENTORY_SLOT_SIZE - 8.0),
                                        height: Val::Px(INVENTORY_SLOT_SIZE - 8.0),
                                        ..default()
                                    },
                                ));
                            }

                            slot.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(3.0),
                                    top: Val::Px(3.0),
                                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.12, 0.10, 0.08, 0.92)),
                            ))
                            .with_children(|badge| {
                                badge.spawn((
                                    Text::new(item.quantity.to_string()),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        }
                    });
            }
        });
}

fn spawn_player_bar<T: Component>(
    commands: &mut Commands,
    empty_texture: Handle<Image>,
    fill_texture: Handle<Image>,
    top: f32,
    fill_ratio: f32,
    fill_marker: T,
) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(top),
            left: Val::Px(PLAYER_BAR_LEFT),
            width: Val::Px(PLAYER_BAR_WIDTH),
            height: Val::Px(PLAYER_BAR_HEIGHT),
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                ImageNode::new(empty_texture),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Px(PLAYER_BAR_WIDTH),
                    height: Val::Px(PLAYER_BAR_HEIGHT),
                    ..default()
                },
            ));

            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Px(bar_fill_width(fill_ratio)),
                    height: Val::Px(PLAYER_BAR_HEIGHT),
                    overflow: Overflow::clip(),
                    ..default()
                },
                fill_marker,
            ))
            .with_children(|fill| {
                fill.spawn((
                    ImageNode::new(fill_texture),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Px(PLAYER_BAR_WIDTH),
                        height: Val::Px(PLAYER_BAR_HEIGHT),
                        ..default()
                    },
                ));
            });
        });
}

fn set_bar_fill_ratio(node: &mut Node, ratio: f32) {
    node.width = Val::Px(bar_fill_width(ratio));
}

fn bar_fill_width(ratio: f32) -> f32 {
    PLAYER_BAR_WIDTH * ratio.clamp(0.0, 1.0)
}

fn hud_value_ratio(value: u32) -> f32 {
    (value as f32 / PLAYER_BAR_MAX_VALUE).clamp(0.0, 1.0)
}
