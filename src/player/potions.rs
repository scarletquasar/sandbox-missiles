use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    player::{
        inventory::{Effect, Inventory, ItemCatalog},
        Player,
    },
    sprites::pastoral_tileset::TILE_WORLD_SIZE,
};

const MISSILE_SKY_OFFSET: f32 = 420.0;
const MISSILE_FALL_SPEED: f32 = 620.0;
const MISSILE_SIZE: Vec2 = Vec2::new(10.0, 34.0);
const IMPACT_DURATION_SECONDS: f32 = 0.45;
const ITEM_USE_ENERGY_COST: f32 = 10.0;

#[derive(Resource, Default)]
pub struct SelectedInventorySlot {
    pub slot_index: Option<usize>,
}

#[derive(Component)]
pub struct PotionMissile {
    target: Vec3,
    range_tiles: f32,
    impact_color: Color,
}

#[derive(Component)]
pub struct PotionImpact {
    timer: Timer,
    duration_seconds: f32,
}

pub fn select_inventory_item(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<Inventory>,
    mut selected_slot: ResMut<SelectedInventorySlot>,
) {
    let Some(slot_index) = pressed_inventory_slot(keyboard.as_ref()) else {
        return;
    };

    selected_slot.slot_index = inventory.items.get(slot_index).map(|_| slot_index);
}

pub fn drop_selected_item_effect(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    item_catalog: Res<ItemCatalog>,
    mut inventory: ResMut<Inventory>,
    mut selected_slot: ResMut<SelectedInventorySlot>,
    mut player_query: Query<&mut Player>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(slot_index) = selected_slot.slot_index else {
        return;
    };

    let Some(item_id) = inventory
        .items
        .get(slot_index)
        .map(|item| item.item_id.clone())
    else {
        selected_slot.slot_index = None;
        return;
    };

    let Some(item) = item_catalog.items.get(&item_id) else {
        selected_slot.slot_index = None;
        return;
    };

    let Some(Effect::DamageToEnemies(damage, range_tiles)) =
        item_catalog.effects.get(&item.effect_id)
    else {
        selected_slot.slot_index = None;
        return;
    };

    let Some(target) = cursor_world_position(&windows, &camera_query) else {
        return;
    };

    let Ok(mut player) = player_query.get_single_mut() else {
        return;
    };

    if !player.try_spend_energy(ITEM_USE_ENERGY_COST) {
        return;
    };

    spawn_potion_missile(
        &mut commands,
        Vec3::new(target.x, target.y, 0.75),
        &item.id,
        *damage,
        *range_tiles,
    );

    let _ = inventory.remove_item(&item_id, 1);
    selected_slot.slot_index = None;
}

pub fn animate_potion_missiles(
    mut commands: Commands,
    time: Res<Time>,
    mut missiles: Query<(Entity, &mut Transform, &PotionMissile)>,
) {
    for (entity, mut transform, missile) in &mut missiles {
        transform.translation.y -= MISSILE_FALL_SPEED * time.delta_secs();

        if transform.translation.y > missile.target.y {
            continue;
        }

        commands.entity(entity).despawn();
        spawn_potion_impact(
            &mut commands,
            missile.target,
            missile.range_tiles,
            missile.impact_color.clone(),
        );
    }
}

pub fn animate_potion_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut impacts: Query<(Entity, &mut Transform, &mut PotionImpact)>,
) {
    for (entity, mut transform, mut impact) in &mut impacts {
        impact.timer.tick(time.delta());

        let progress =
            (impact.timer.elapsed().as_secs_f32() / impact.duration_seconds).clamp(0.0, 1.0);
        let scale = 0.25 + progress * 0.75;
        transform.scale = Vec3::splat(scale);
        transform.rotation = Quat::from_rotation_z(progress * std::f32::consts::FRAC_PI_2);

        if impact.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn pressed_inventory_slot(keyboard: &ButtonInput<KeyCode>) -> Option<usize> {
    [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ]
    .into_iter()
    .position(|key_code| keyboard.just_pressed(key_code))
}

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let cursor_position = window.cursor_position()?;
    let (camera, camera_transform) = camera_query.get_single().ok()?;

    camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()
}

fn spawn_potion_missile(
    commands: &mut Commands,
    target: Vec3,
    item_id: &str,
    damage: f32,
    range_tiles: f32,
) {
    let visual = potion_visual(item_id);
    let clamped_range_tiles = range_tiles.clamp(1.0, 9.0);
    let spawn_position = Vec3::new(target.x, target.y + MISSILE_SKY_OFFSET, target.z + 10.0);
    let target = Vec3::new(target.x, target.y, target.z + 5.0);

    commands.spawn((
        Name::new(format!("{item_id} Missile")),
        Sprite {
            color: visual.missile_color,
            custom_size: Some(MISSILE_SIZE),
            ..default()
        },
        Transform::from_translation(spawn_position)
            .with_rotation(Quat::from_rotation_z(-0.22))
            .with_scale(Vec3::splat(1.0 + damage.max(0.0) * 0.002)),
        PotionMissile {
            target,
            range_tiles: clamped_range_tiles,
            impact_color: visual.impact_color,
        },
    ));
}

fn spawn_potion_impact(commands: &mut Commands, target: Vec3, range_tiles: f32, color: Color) {
    let impact_size = Vec2::splat(range_tiles * TILE_WORLD_SIZE);

    commands.spawn((
        Name::new("Potion Missile Impact"),
        Sprite {
            color,
            custom_size: Some(impact_size),
            ..default()
        },
        Transform::from_translation(Vec3::new(target.x, target.y, target.z - 0.1))
            .with_scale(Vec3::splat(0.25)),
        PotionImpact {
            timer: Timer::from_seconds(IMPACT_DURATION_SECONDS, TimerMode::Once),
            duration_seconds: IMPACT_DURATION_SECONDS,
        },
    ));
}

struct PotionVisual {
    missile_color: Color,
    impact_color: Color,
}

fn potion_visual(item_id: &str) -> PotionVisual {
    match item_id {
        "potion_red" => PotionVisual {
            missile_color: Color::srgb(1.0, 0.18, 0.08),
            impact_color: Color::srgba(1.0, 0.18, 0.08, 0.45),
        },
        "potion_green" => PotionVisual {
            missile_color: Color::srgb(0.2, 0.95, 0.28),
            impact_color: Color::srgba(0.2, 0.95, 0.28, 0.42),
        },
        "potion_blue" => PotionVisual {
            missile_color: Color::srgb(0.2, 0.5, 1.0),
            impact_color: Color::srgba(0.2, 0.5, 1.0, 0.42),
        },
        "potion_black" => PotionVisual {
            missile_color: Color::srgb(0.12, 0.04, 0.22),
            impact_color: Color::srgba(0.55, 0.12, 0.9, 0.5),
        },
        _ => PotionVisual {
            missile_color: Color::srgb(1.0, 0.95, 0.4),
            impact_color: Color::srgba(1.0, 0.95, 0.4, 0.4),
        },
    }
}
