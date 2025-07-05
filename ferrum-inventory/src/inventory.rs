use crate::{ItemStack, Slot};

const INVENTORY_SIZE: usize = 36;

pub struct Inventory {
    slots: [Slot; INVENTORY_SIZE],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: [Slot::new(); INVENTORY_SIZE],
        }
    }

    pub fn add_item(&mut self, mut item: ItemStack) -> bool {
        if item.count == 0 {
            return true;
        }

        for slot in &mut self.slots {
            if let Some(existing) = &mut slot.item {
                if existing.can_stack_with(&item) && !existing.is_full() {
                    let space = existing.remaining_space();
                    let to_add = item.count.min(space);
                    existing.count += to_add;
                    item.count -= to_add;

                    if item.count == 0 {
                        return true;
                    }
                }
            }
        }

        for slot in &mut self.slots {
            if slot.is_empty() {
                slot.item = Some(item);
                return true;
            }
        }

        false
    }

    pub fn remove_item(&mut self, slot: usize) -> Option<ItemStack> {
        if slot >= INVENTORY_SIZE {
            return None;
        }

        let item = self.slots[slot].item.take();
        item
    }

    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        if from >= INVENTORY_SIZE || to >= INVENTORY_SIZE || from == to {
            return false;
        }

        let from_item = self.slots[from].item;
        let to_item = self.slots[to].item;

        match (from_item, to_item) {
            (Some(from_stack), Some(mut to_stack)) => {
                if from_stack.can_stack_with(&to_stack) {
                    let space = to_stack.remaining_space();
                    let to_add = from_stack.count.min(space);
                    to_stack.count += to_add;

                    let remaining = from_stack.count - to_add;
                    if remaining > 0 {
                        self.slots[from].item = Some(ItemStack::new(
                            from_stack.item_id,
                            remaining,
                            from_stack.max_stack_size,
                        ));
                    } else {
                        self.slots[from].item = None;
                    }

                    self.slots[to].item = Some(to_stack);
                } else {
                    self.slots[from].item = to_item;
                    self.slots[to].item = from_item;
                }
                true
            }
            (Some(_), None) => {
                self.slots[to].item = from_item;
                self.slots[from].item = None;
                true
            }
            _ => false,
        }
    }

    pub fn find_item(&self, item_id: u16) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| slot.item.map_or(false, |item| item.item_id == item_id))
    }

    pub fn get_slot(&self, index: usize) -> Option<&Slot> {
        self.slots.get(index)
    }

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Slot> {
        self.slots.get_mut(index)
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
