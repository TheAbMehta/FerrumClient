use ferrum_inventory::{CraftingTable, ItemStack, Recipe};

const PLANKS: u16 = 1;
const STICK: u16 = 2;
const STONE: u16 = 3;
const PICKAXE: u16 = 4;
const SWORD: u16 = 5;

#[test]
fn test_crafting_table_creation() {
    let table = CraftingTable::new();
    assert!(table.is_empty());
}

#[test]
fn test_set_ingredient() {
    let mut table = CraftingTable::new();
    let planks = ItemStack::new(PLANKS, 1, 64);
    table.set_ingredient(0, 0, Some(planks));

    assert_eq!(table.get_ingredient(0, 0), Some(planks));
}

#[test]
fn test_clear_grid() {
    let mut table = CraftingTable::new();
    let planks = ItemStack::new(PLANKS, 1, 64);
    table.set_ingredient(0, 0, Some(planks));
    table.clear();

    assert!(table.is_empty());
}

#[test]
fn test_shaped_recipe_sticks() {
    // Recipe: 2 planks vertical = 4 sticks
    // Pattern:
    // [P][ ][ ]
    // [P][ ][ ]
    // [ ][ ][ ]

    let pattern = [
        [Some(PLANKS), None, None],
        [Some(PLANKS), None, None],
        [None, None, None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(STICK, 4, 64));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 0, Some(ItemStack::new(PLANKS, 1, 64)));
    table.set_ingredient(1, 0, Some(ItemStack::new(PLANKS, 1, 64)));

    assert!(recipe.matches(&table));
}

#[test]
fn test_shaped_recipe_pickaxe() {
    // Recipe: 3 stone top row, 2 sticks middle/bottom center = pickaxe
    // Pattern:
    // [S][S][S]
    // [ ][T][ ]
    // [ ][T][ ]

    let pattern = [
        [Some(STONE), Some(STONE), Some(STONE)],
        [None, Some(STICK), None],
        [None, Some(STICK), None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(PICKAXE, 1, 1));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 0, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(0, 1, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(0, 2, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(1, 1, Some(ItemStack::new(STICK, 1, 64)));
    table.set_ingredient(2, 1, Some(ItemStack::new(STICK, 1, 64)));

    assert!(recipe.matches(&table));
}

#[test]
fn test_recipe_mismatch() {
    let pattern = [
        [Some(STONE), Some(STONE), Some(STONE)],
        [None, Some(STICK), None],
        [None, Some(STICK), None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(PICKAXE, 1, 1));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 0, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(0, 1, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(0, 2, Some(ItemStack::new(STONE, 1, 64)));

    assert!(!recipe.matches(&table));
}

#[test]
fn test_craft_sticks_consumes_ingredients() {
    let pattern = [
        [Some(PLANKS), None, None],
        [Some(PLANKS), None, None],
        [None, None, None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(STICK, 4, 64));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 0, Some(ItemStack::new(PLANKS, 2, 64)));
    table.set_ingredient(1, 0, Some(ItemStack::new(PLANKS, 3, 64)));

    let result = table.craft(&recipe);

    assert_eq!(result, Some(ItemStack::new(STICK, 4, 64)));
    assert_eq!(
        table.get_ingredient(0, 0),
        Some(ItemStack::new(PLANKS, 1, 64))
    );
    assert_eq!(
        table.get_ingredient(1, 0),
        Some(ItemStack::new(PLANKS, 2, 64))
    );
}

#[test]
fn test_craft_pickaxe_consumes_all() {
    let pattern = [
        [Some(STONE), Some(STONE), Some(STONE)],
        [None, Some(STICK), None],
        [None, Some(STICK), None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(PICKAXE, 1, 1));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 0, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(0, 1, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(0, 2, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(1, 1, Some(ItemStack::new(STICK, 1, 64)));
    table.set_ingredient(2, 1, Some(ItemStack::new(STICK, 1, 64)));

    let result = table.craft(&recipe);

    assert_eq!(result, Some(ItemStack::new(PICKAXE, 1, 1)));
    assert_eq!(table.get_ingredient(0, 0), None);
    assert_eq!(table.get_ingredient(0, 1), None);
    assert_eq!(table.get_ingredient(0, 2), None);
    assert_eq!(table.get_ingredient(1, 1), None);
    assert_eq!(table.get_ingredient(2, 1), None);
}

#[test]
fn test_craft_fails_when_no_match() {
    let pattern = [
        [Some(STONE), Some(STONE), Some(STONE)],
        [None, Some(STICK), None],
        [None, Some(STICK), None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(PICKAXE, 1, 1));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 0, Some(ItemStack::new(PLANKS, 1, 64)));

    let result = table.craft(&recipe);

    assert_eq!(result, None);
    assert_eq!(
        table.get_ingredient(0, 0),
        Some(ItemStack::new(PLANKS, 1, 64))
    );
}

#[test]
fn test_empty_grid_no_result() {
    let pattern = [
        [Some(PLANKS), None, None],
        [Some(PLANKS), None, None],
        [None, None, None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(STICK, 4, 64));

    let table = CraftingTable::new();

    assert!(!recipe.matches(&table));
}

#[test]
fn test_sword_recipe() {
    // Recipe: 2 material vertical, 1 stick at bottom = sword
    // Pattern:
    // [ ][S][ ]
    // [ ][S][ ]
    // [ ][T][ ]

    let pattern = [
        [None, Some(STONE), None],
        [None, Some(STONE), None],
        [None, Some(STICK), None],
    ];

    let recipe = Recipe::shaped(pattern, ItemStack::new(SWORD, 1, 1));

    let mut table = CraftingTable::new();
    table.set_ingredient(0, 1, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(1, 1, Some(ItemStack::new(STONE, 1, 64)));
    table.set_ingredient(2, 1, Some(ItemStack::new(STICK, 1, 64)));

    assert!(recipe.matches(&table));

    let result = table.craft(&recipe);
    assert_eq!(result, Some(ItemStack::new(SWORD, 1, 1)));
}
