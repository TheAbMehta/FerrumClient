use crate::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub item: Option<ItemStack>,
}

impl Slot {
    pub fn new() -> Self {
        Self { item: None }
    }

    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    pub fn clear(&mut self) {
        self.item = None;
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::new()
    }
}
