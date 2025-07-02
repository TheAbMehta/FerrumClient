use ferrum_inventory::{Inventory, ItemStack};

#[test]
fn test_inventory_creation() {
    let inventory = Inventory::new();
    for i in 0..36 {
        assert!(inventory.get_slot(i).unwrap().is_empty());
    }
}

#[test]
fn test_itemstack_creation() {
    let stone = ItemStack::new(1, 32, 64);
    assert_eq!(stone.item_id, 1);
    assert_eq!(stone.count, 32);
    assert_eq!(stone.max_stack_size, 64);
}

#[test]
fn test_itemstack_can_stack() {
    let stone1 = ItemStack::new(1, 32, 64);
    let stone2 = ItemStack::new(1, 16, 64);
    let dirt = ItemStack::new(2, 32, 64);

    assert!(stone1.can_stack_with(&stone2));
    assert!(!stone1.can_stack_with(&dirt));
}

#[test]
fn test_itemstack_remaining_space() {
    let stone = ItemStack::new(1, 32, 64);
    assert_eq!(stone.remaining_space(), 32);

    let full_stone = ItemStack::new(1, 64, 64);
    assert_eq!(full_stone.remaining_space(), 0);
}

#[test]
fn test_itemstack_is_full() {
    let stone = ItemStack::new(1, 32, 64);
    assert!(!stone.is_full());

    let full_stone = ItemStack::new(1, 64, 64);
    assert!(full_stone.is_full());
}

#[test]
fn test_add_item_to_empty_slot() {
    let mut inventory = Inventory::new();
    let stone = ItemStack::new(1, 32, 64);

    assert!(inventory.add_item(stone));
    assert_eq!(inventory.get_slot(0).unwrap().item, Some(stone));
}

#[test]
fn test_add_item_stacks_with_existing() {
    let mut inventory = Inventory::new();
    let stone1 = ItemStack::new(1, 32, 64);
    let stone2 = ItemStack::new(1, 16, 64);

    inventory.add_item(stone1);
    inventory.add_item(stone2);

    let slot = inventory.get_slot(0).unwrap();
    assert_eq!(slot.item.unwrap().count, 48);
}

#[test]
fn test_add_item_overflow_creates_new_stack() {
    let mut inventory = Inventory::new();
    let stone1 = ItemStack::new(1, 60, 64);
    let stone2 = ItemStack::new(1, 10, 64);

    inventory.add_item(stone1);
    inventory.add_item(stone2);

    assert_eq!(inventory.get_slot(0).unwrap().item.unwrap().count, 64);
    assert_eq!(inventory.get_slot(1).unwrap().item.unwrap().count, 6);
}

#[test]
fn test_remove_item() {
    let mut inventory = Inventory::new();
    let stone = ItemStack::new(1, 32, 64);

    inventory.add_item(stone);
    let removed = inventory.remove_item(0);

    assert_eq!(removed, Some(stone));
    assert!(inventory.get_slot(0).unwrap().is_empty());
}

#[test]
fn test_remove_item_from_empty_slot() {
    let mut inventory = Inventory::new();
    let removed = inventory.remove_item(0);
    assert_eq!(removed, None);
}

#[test]
fn test_move_item_between_slots() {
    let mut inventory = Inventory::new();
    let stone = ItemStack::new(1, 32, 64);

    inventory.add_item(stone);
    assert!(inventory.move_item(0, 5));

    assert!(inventory.get_slot(0).unwrap().is_empty());
    assert_eq!(inventory.get_slot(5).unwrap().item, Some(stone));
}

#[test]
fn test_move_item_stacks_if_compatible() {
    let mut inventory = Inventory::new();
    let stone1 = ItemStack::new(1, 32, 64);
    let stone2 = ItemStack::new(1, 16, 64);

    inventory.get_slot_mut(0).unwrap().item = Some(stone1);
    inventory.get_slot_mut(5).unwrap().item = Some(stone2);

    inventory.move_item(0, 5);

    assert!(inventory.get_slot(0).unwrap().is_empty());
    assert_eq!(inventory.get_slot(5).unwrap().item.unwrap().count, 48);
}

#[test]
fn test_move_item_swaps_if_incompatible() {
    let mut inventory = Inventory::new();
    let stone = ItemStack::new(1, 32, 64);
    let dirt = ItemStack::new(2, 16, 64);

    inventory.get_slot_mut(0).unwrap().item = Some(stone);
    inventory.get_slot_mut(5).unwrap().item = Some(dirt);

    inventory.move_item(0, 5);

    assert_eq!(inventory.get_slot(0).unwrap().item, Some(dirt));
    assert_eq!(inventory.get_slot(5).unwrap().item, Some(stone));
}

#[test]
fn test_find_item() {
    let mut inventory = Inventory::new();
    let stone = ItemStack::new(1, 32, 64);

    inventory.get_slot_mut(10).unwrap().item = Some(stone);

    assert_eq!(inventory.find_item(1), Some(10));
    assert_eq!(inventory.find_item(2), None);
}

#[test]
fn test_inventory_full() {
    let mut inventory = Inventory::new();
    let stone = ItemStack::new(1, 64, 64);

    for _ in 0..36 {
        inventory.add_item(stone);
    }

    assert!(!inventory.add_item(stone));
}
