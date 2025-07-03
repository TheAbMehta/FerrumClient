#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub item_id: u16,
    pub count: u8,
    pub max_stack_size: u8,
}

impl ItemStack {
    pub fn new(item_id: u16, count: u8, max_stack_size: u8) -> Self {
        Self {
            item_id,
            count,
            max_stack_size,
        }
    }

    pub fn can_stack_with(&self, other: &ItemStack) -> bool {
        self.item_id == other.item_id && self.max_stack_size == other.max_stack_size
    }

    pub fn remaining_space(&self) -> u8 {
        self.max_stack_size.saturating_sub(self.count)
    }

    pub fn is_full(&self) -> bool {
        self.count >= self.max_stack_size
    }
}
