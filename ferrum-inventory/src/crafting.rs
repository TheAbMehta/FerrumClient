use crate::ItemStack;

pub struct CraftingTable {
    grid: [[Option<ItemStack>; 3]; 3],
}

impl CraftingTable {
    pub fn new() -> Self {
        Self {
            grid: [[None; 3]; 3],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.grid
            .iter()
            .all(|row| row.iter().all(|slot| slot.is_none()))
    }

    pub fn get_ingredient(&self, row: usize, col: usize) -> Option<ItemStack> {
        if row < 3 && col < 3 {
            self.grid[row][col]
        } else {
            None
        }
    }

    pub fn set_ingredient(&mut self, row: usize, col: usize, item: Option<ItemStack>) {
        if row < 3 && col < 3 {
            self.grid[row][col] = item;
        }
    }

    pub fn clear(&mut self) {
        self.grid = [[None; 3]; 3];
    }

    pub fn craft(&mut self, recipe: &Recipe) -> Option<ItemStack> {
        if !recipe.matches(self) {
            return None;
        }

        for row in 0..3 {
            for col in 0..3 {
                if recipe.pattern[row][col].is_some() {
                    if let Some(item) = &mut self.grid[row][col] {
                        item.count -= 1;
                        if item.count == 0 {
                            self.grid[row][col] = None;
                        }
                    }
                }
            }
        }

        Some(recipe.output)
    }
}

impl Default for CraftingTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Recipe {
    pattern: [[Option<u16>; 3]; 3],
    output: ItemStack,
}

impl Recipe {
    pub fn shaped(pattern: [[Option<u16>; 3]; 3], output: ItemStack) -> Self {
        Self { pattern, output }
    }

    pub fn matches(&self, table: &CraftingTable) -> bool {
        for row in 0..3 {
            for col in 0..3 {
                match (self.pattern[row][col], table.grid[row][col]) {
                    (Some(required_id), Some(item)) => {
                        if item.item_id != required_id {
                            return false;
                        }
                    }
                    (Some(_), None) => return false,
                    (None, Some(_)) => return false,
                    (None, None) => {}
                }
            }
        }
        true
    }
}
