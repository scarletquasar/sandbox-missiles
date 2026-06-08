use bevy::{prelude::Resource, utils::HashMap};

// ===== Items and effects that can be used by the player =====
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effect_id: String,
    pub sprite_path: String,
}

#[derive(Clone)]
pub enum Effect {
    DamageToEnemies(f32, f32), // damage and range in tiles [1 - 9] always square
    Heal(f32),                 // amount of healing, cant exceed player's max health
    Slow(f32, f32),            // percentage to slow and duration in seconds
    Speed(f32, f32),           // percentage to speed up and duration in seconds
    Teleport(String),          // destination in the format "x,y"
    Voyage(String, f32),       // scene name and fade duration
}

// ====== Specific for each inventory =======

pub struct InventoryItem {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Resource)]
pub struct Inventory {
    pub max_inventory_size: usize,
    pub items: Vec<InventoryItem>,
}

#[derive(Resource, Default)]
pub struct ItemCatalog {
    pub items: HashMap<String, Item>,
    pub effects: HashMap<String, Effect>,
}

pub fn create_inventory(max_inventory_size: usize) -> Inventory {
    Inventory {
        max_inventory_size,
        items: Vec::new(),
    }
}

pub fn create_item(
    id: String,
    name: String,
    description: String,
    effect_id: String,
    sprite_path: String,
) -> Item {
    Item {
        id,
        name,
        description,
        effect_id,
        sprite_path,
    }
}

impl Inventory {
    pub fn add_item(
        &mut self,
        item_catalog: &ItemCatalog,
        item_id: String,
        quantity: u32,
    ) -> Result<(), String> {
        if !item_catalog.items.contains_key(&item_id) {
            return Err(format!(
                "Item '{item_id}' does not exist in the item catalog"
            ));
        }

        if self.items.len() >= self.max_inventory_size {
            return Err("Inventory is full".to_owned());
        }

        if let Some(existing_item) = self.items.iter_mut().find(|item| item.item_id == item_id) {
            existing_item.quantity += quantity;
        } else {
            self.items.push(InventoryItem { item_id, quantity });
        }

        Ok(())
    }

    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> Result<(), String> {
        if let Some(existing_item) = self.items.iter_mut().find(|item| item.item_id == item_id) {
            if existing_item.quantity < quantity {
                return Err("Not enough quantity to remove".to_owned());
            }
            existing_item.quantity -= quantity;
            if existing_item.quantity == 0 {
                self.items.retain(|item| item.item_id != item_id);
            }
            Ok(())
        } else {
            Err("Item not found in inventory".to_owned())
        }
    }

    pub fn change_items_position(
        &mut self,
        item_id: &str,
        new_position: usize,
    ) -> Result<(), String> {
        if new_position >= self.items.len() {
            return Err("New position is out of bounds".to_owned());
        }

        if let Some(index) = self.items.iter().position(|item| item.item_id == item_id) {
            let item = self.items.remove(index);
            self.items.insert(new_position, item);
            Ok(())
        } else {
            Err("Item not found in inventory".to_owned())
        }
    }
}
