use crate::world::Block;

pub const INVENTORY_SIZE: usize = 46;
pub const HOTBAR_START: usize = 36;
pub const HOTBAR_LEN: usize = 9;
pub const MAX_HOTBAR_INDEX: u8 = 8;
pub const MAX_STACK: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Air,
    Stone,
    Dirt,
    GrassBlock,
}

impl Item {
    #[must_use]
    pub fn id(self) -> i32 {
        match self {
            Item::Air => 0,
            Item::Stone => 1,
            Item::GrassBlock => 27,
            Item::Dirt => 28,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Item::Air => "minecraft:air",
            Item::Stone => "minecraft:stone",
            Item::Dirt => "minecraft:dirt",
            Item::GrassBlock => "minecraft:grass_block",
        }
    }

    #[must_use]
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Item::Air),
            1 => Some(Item::Stone),
            27 => Some(Item::GrassBlock),
            28 => Some(Item::Dirt),
            _ => None,
        }
    }

    #[must_use]
    pub fn block(self) -> Option<Block> {
        match self {
            Item::Air => None,
            Item::Stone => Some(Block::Stone),
            Item::Dirt => Some(Block::Dirt),
            Item::GrassBlock => Some(Block::GrassBlock),
        }
    }

    #[must_use]
    pub fn max_stack(self) -> u32 {
        let _ = self;
        MAX_STACK
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            item: Item::Air,
            count: 0,
        }
    }

    #[must_use]
    pub fn new(item: Item, count: u32) -> Option<Self> {
        if item == Item::Air || count == 0 || count > item.max_stack() {
            return None;
        }
        Some(Self { item, count })
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.item == Item::Air || self.count == 0
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    slots: [ItemStack; INVENTORY_SIZE],
    selected: u8,
}

impl Inventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [ItemStack::empty(); INVENTORY_SIZE],
            selected: 0,
        }
    }

    #[must_use]
    pub fn starting() -> Self {
        let mut inventory = Self::new();
        inventory.slots[HOTBAR_START] = ItemStack::new(Item::Stone, 64).expect("valid stack");
        inventory.slots[HOTBAR_START + 1] = ItemStack::new(Item::Dirt, 64).expect("valid stack");
        inventory.slots[HOTBAR_START + 2] =
            ItemStack::new(Item::GrassBlock, 64).expect("valid stack");
        inventory
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<ItemStack> {
        self.slots.get(index).copied()
    }

    pub fn set(&mut self, index: usize, stack: ItemStack) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                *slot = stack;
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self, index: usize) -> bool {
        self.set(index, ItemStack::empty())
    }

    pub fn add(&mut self, mut stack: ItemStack) -> u32 {
        if stack.is_empty() {
            return 0;
        }
        for slot in self.slots.iter_mut() {
            if stack.count == 0 {
                break;
            }
            if slot.item == stack.item && slot.count < slot.item.max_stack() {
                let room = slot.item.max_stack() - slot.count;
                let take = room.min(stack.count);
                slot.count += take;
                stack.count -= take;
            }
        }
        for slot in self.slots.iter_mut() {
            if stack.count == 0 {
                break;
            }
            if slot.is_empty() {
                let take = stack.item.max_stack().min(stack.count);
                *slot = ItemStack {
                    item: stack.item,
                    count: take,
                };
                stack.count -= take;
            }
        }
        stack.count
    }

    pub fn take(&mut self, index: usize, count: u32) -> Option<ItemStack> {
        let slot = self.slots.get_mut(index)?;
        if slot.is_empty() || count == 0 {
            return None;
        }
        let take = count.min(slot.count);
        let taken = ItemStack {
            item: slot.item,
            count: take,
        };
        slot.count -= take;
        if slot.count == 0 {
            *slot = ItemStack::empty();
        }
        Some(taken)
    }

    pub fn consume_one(&mut self, index: usize) -> bool {
        self.take(index, 1).is_some()
    }

    #[must_use]
    pub fn selected(&self) -> u8 {
        self.selected
    }

    pub fn select(&mut self, index: u8) -> bool {
        if index > MAX_HOTBAR_INDEX {
            return false;
        }
        self.selected = index;
        true
    }

    #[must_use]
    pub fn selected_slot_index(&self) -> usize {
        HOTBAR_START + self.selected as usize
    }

    #[must_use]
    pub fn held(&self) -> ItemStack {
        self.slots[self.selected_slot_index()]
    }

    #[must_use]
    pub fn hotbar(&self, index: u8) -> Option<ItemStack> {
        if index > MAX_HOTBAR_INDEX {
            return None;
        }
        Some(self.slots[HOTBAR_START + index as usize])
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_ids_match_vanilla_registry() {
        assert_eq!(Item::Air.id(), 0);
        assert_eq!(Item::Stone.id(), 1);
        assert_eq!(Item::GrassBlock.id(), 27);
        assert_eq!(Item::Dirt.id(), 28);
        assert_eq!(Item::from_id(1), Some(Item::Stone));
        assert_eq!(Item::from_id(99), None);
    }

    #[test]
    fn items_map_to_matching_blocks() {
        assert_eq!(Item::Stone.block(), Some(Block::Stone));
        assert_eq!(Item::Dirt.block(), Some(Block::Dirt));
        assert_eq!(Item::GrassBlock.block(), Some(Block::GrassBlock));
        assert_eq!(Item::Air.block(), None);
        assert_eq!(Item::Stone.name(), Block::Stone.name());
    }

    #[test]
    fn stacks_reject_invalid_counts() {
        assert!(ItemStack::new(Item::Stone, 0).is_none());
        assert!(ItemStack::new(Item::Stone, 65).is_none());
        assert!(ItemStack::new(Item::Air, 5).is_none());
        let stack = ItemStack::new(Item::Dirt, 64).unwrap();
        assert!(!stack.is_empty());
        assert!(ItemStack::empty().is_empty());
    }

    #[test]
    fn inventory_reads_writes_and_clears() {
        let mut inventory = Inventory::new();
        assert_eq!(inventory.get(0), Some(ItemStack::empty()));
        assert_eq!(inventory.get(99), None);
        assert!(inventory.set(9, ItemStack::new(Item::Stone, 5).unwrap()));
        assert_eq!(
            inventory.get(9),
            Some(ItemStack::new(Item::Stone, 5).unwrap())
        );
        assert!(!inventory.set(99, ItemStack::empty()));
        assert!(inventory.clear(9));
        assert_eq!(inventory.get(9), Some(ItemStack::empty()));
        assert!(!inventory.clear(99));
    }

    #[test]
    fn inventory_adds_merges_and_reports_leftover() {
        let mut inventory = Inventory::new();
        inventory.set(0, ItemStack::new(Item::Stone, 60).unwrap());
        assert_eq!(inventory.add(ItemStack::new(Item::Stone, 10).unwrap()), 0);
        assert_eq!(inventory.get(0).unwrap().count, 64);
        assert_eq!(inventory.get(1).unwrap().count, 6);

        let mut full = Inventory::new();
        for index in 0..INVENTORY_SIZE {
            full.set(index, ItemStack::new(Item::Dirt, 64).unwrap());
        }
        assert_eq!(full.add(ItemStack::new(Item::Dirt, 5).unwrap()), 5);
        assert_eq!(full.add(ItemStack::empty()), 0);
    }

    #[test]
    fn inventory_take_consumes_and_empties() {
        let mut inventory = Inventory::new();
        inventory.set(3, ItemStack::new(Item::Dirt, 5).unwrap());
        assert_eq!(
            inventory.take(3, 2),
            Some(ItemStack::new(Item::Dirt, 2).unwrap())
        );
        assert_eq!(inventory.get(3).unwrap().count, 3);
        assert!(inventory.consume_one(3));
        assert_eq!(inventory.take(99, 1), None);
        assert_eq!(inventory.take(4, 1), None);
        assert_eq!(inventory.take(3, 0), None);
    }

    #[test]
    fn hotbar_selection_drives_held_item() {
        let mut inventory = Inventory::starting();
        assert_eq!(inventory.selected(), 0);
        assert_eq!(inventory.hotbar(0).unwrap().item, Item::Stone);
        assert_eq!(inventory.hotbar(8), Some(ItemStack::empty()));
        assert_eq!(inventory.hotbar(9), None);
        assert_eq!(inventory.held().item, Item::Stone);
        assert!(inventory.select(2));
        assert_eq!(inventory.held().item, Item::GrassBlock);
        assert!(!inventory.select(9));
        assert!(!inventory.select(u8::MAX));
        assert_eq!(inventory.selected(), 2);
    }
}
