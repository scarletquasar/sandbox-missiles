use bevy::{prelude::*, utils::HashMap};

pub struct HudData {
    pub health: u32,
    pub energy: u32,
}

pub fn display_hud(commands: &mut Commands, hud_data: &HudData) -> HashMap<String, Entity> {
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

    spawned_texts
}
