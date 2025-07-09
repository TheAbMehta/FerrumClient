//! Inventory system for Ferrum Minecraft Client
//!
//! Provides inventory storage, item stacks, and slot management.
//!
//! # Example
//! ```
//! use ferrum_inventory::{Inventory, ItemStack};
//!
//! let mut inventory = Inventory::new();
//! let stone = ItemStack::new(1, 64, 64); // Stone with max stack 64
//! inventory.add_item(stone);
//! ```

mod combat;
mod crafting;
mod inventory;
mod item_stack;
mod slot;

pub use combat::{attack, Health, Weapon};
pub use crafting::{CraftingTable, Recipe};
pub use inventory::Inventory;
pub use item_stack::ItemStack;
pub use slot::Slot;
