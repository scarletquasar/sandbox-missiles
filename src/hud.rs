use bevy::{prelude::*, utils::HashMap};

use crate::player::inventory::{Inventory, ItemCatalog};

pub struct HudData {
    pub health: u32,
    pub energy: u32,
}

const INVENTORY_SLOT_SIZE: f32 = 56.0;
const INVENTORY_SLOT_GAP: f32 = 6.0;

pub fn display_hud(
    commands: &mut Commands,
    asset_server: &AssetServer,
    inventory: &Inventory,
    item_catalog: &ItemCatalog,
    hud_data: &HudData,
) -> HashMap<String, Entity> {
    let mut spawned_texts = HashMap::new();

    let health_entity = commands
        .spawn((
            Text::new(format!("Health: {}", hud_data.health)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
        ))
        .id();

    let energy_entity = commands
        .spawn((
            Text::new(format!("Energy: {}", hud_data.energy)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(36.0),
                left: Val::Px(12.0),
                ..default()
            },
        ))
        .id();

    spawned_texts.insert("health".to_string(), health_entity);
    spawned_texts.insert("energy".to_string(), energy_entity);

    let inventory_visual_entity = commands
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
        ))
        .with_children(|parent| {
            for slot_index in 0..inventory.max_inventory_size {
                let item = inventory.items.get(slot_index);

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
                        BackgroundColor(Color::srgba(0.98, 0.97, 0.92, 0.96)),
                        BorderColor(Color::srgb(0.28, 0.24, 0.18)),
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
        })
        .id();

    spawned_texts.insert("inventory".to_string(), inventory_visual_entity);

    spawned_texts
}
