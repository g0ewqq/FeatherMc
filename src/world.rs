use std::collections::HashMap;

use crate::entity::Entity;
use crate::inventory::ItemStack;

pub const DEFAULT_WORLD_NAME: &str = "world";
pub const DEFAULT_DIMENSION_ID: &str = "minecraft:overworld";

pub const WORLD_MIN_Y: i32 = -64;
pub const WORLD_HEIGHT: i32 = 384;
pub const SECTION_SIZE: i32 = 16;
pub const SECTION_COUNT: usize = (WORLD_HEIGHT / SECTION_SIZE) as usize;
pub const CHUNK_WIDTH: i32 = 16;

pub const SPAWN_X: f64 = 0.0;
pub const SPAWN_Y: f64 = 100.0;
pub const SPAWN_Z: f64 = 0.0;
pub const SPAWN_YAW: f32 = 0.0;
pub const SPAWN_PITCH: f32 = 0.0;

pub const SURFACE_Y: i32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    GrassBlock = 3,
}

impl Block {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Block::Air => "minecraft:air",
            Block::Stone => "minecraft:stone",
            Block::Dirt => "minecraft:dirt",
            Block::GrassBlock => "minecraft:grass_block",
        }
    }

    #[must_use]
    pub fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(Block::Air),
            1 => Some(Block::Stone),
            2 => Some(Block::Dirt),
            3 => Some(Block::GrassBlock),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn chunk_pos(self) -> ChunkPos {
        ChunkPos::from_world(self.x, self.z)
    }

    #[must_use]
    pub fn local_x(self) -> u8 {
        self.x.rem_euclid(CHUNK_WIDTH) as u8
    }

    #[must_use]
    pub fn local_z(self) -> u8 {
        self.z.rem_euclid(CHUNK_WIDTH) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[must_use]
    pub fn from_world(world_x: i32, world_z: i32) -> Self {
        Self {
            x: world_x.div_euclid(CHUNK_WIDTH),
            z: world_z.div_euclid(CHUNK_WIDTH),
        }
    }

    #[must_use]
    pub fn min_block(self) -> (i32, i32) {
        (self.x * CHUNK_WIDTH, self.z * CHUNK_WIDTH)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spawn {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub dimension: &'static str,
}

impl Spawn {
    #[must_use]
    pub fn default_spawn() -> Self {
        Self {
            x: SPAWN_X,
            y: SPAWN_Y,
            z: SPAWN_Z,
            yaw: SPAWN_YAW,
            pitch: SPAWN_PITCH,
            dimension: DEFAULT_DIMENSION_ID,
        }
    }
}

pub struct ChunkSection {
    section_y: i32,
    blocks: Box<[u16; 4096]>,
    changed: Vec<(u8, u8, u8)>,
}

impl ChunkSection {
    #[must_use]
    pub fn new(section_y: i32) -> Self {
        Self {
            section_y,
            blocks: Box::new([Block::Air as u16; 4096]),
            changed: Vec::new(),
        }
    }

    #[must_use]
    pub fn section_y(&self) -> i32 {
        self.section_y
    }

    fn index(x: u8, y: u8, z: u8) -> usize {
        (y as usize * 16 + z as usize) * 16 + x as usize
    }

    #[must_use]
    pub fn get_local(&self, x: u8, y: u8, z: u8) -> Block {
        debug_assert!(x < 16 && y < 16 && z < 16);
        Block::from_id(self.blocks[Self::index(x, y, z)]).unwrap_or(Block::Air)
    }

    pub fn set_local(&mut self, x: u8, y: u8, z: u8, block: Block) -> Block {
        debug_assert!(x < 16 && y < 16 && z < 16);
        let slot = &mut self.blocks[Self::index(x, y, z)];
        let prev = Block::from_id(*slot).unwrap_or(Block::Air);
        if prev != block {
            *slot = block as u16;
            if !self.changed.contains(&(x, y, z)) {
                self.changed.push((x, y, z));
            }
        }
        prev
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.changed.is_empty()
    }

    pub fn take_changed(&mut self) -> Vec<(u8, u8, u8)> {
        std::mem::take(&mut self.changed)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&id| id == Block::Air as u16)
    }

    #[must_use]
    pub fn states(&self) -> &[u16; 4096] {
        &self.blocks
    }
}

pub struct Chunk {
    pos: ChunkPos,
    sections: Vec<ChunkSection>,
}

impl Chunk {
    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        let mut chunk = Self {
            pos: ChunkPos::new(x, z),
            sections: (0..SECTION_COUNT as i32).map(ChunkSection::new).collect(),
        };
        chunk.generate_flat();
        for section in &mut chunk.sections {
            section.take_changed();
        }
        chunk
    }

    pub fn section_mut(&mut self, index: usize) -> Option<&mut ChunkSection> {
        self.sections.get_mut(index)
    }

    #[must_use]
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    fn section_for_y(&self, y: i32) -> Option<usize> {
        let rel = y - WORLD_MIN_Y;
        if !(0..WORLD_HEIGHT).contains(&rel) {
            return None;
        }
        Some((rel / SECTION_SIZE) as usize)
    }

    fn generate_flat(&mut self) {
        for y in WORLD_MIN_Y..=SURFACE_Y {
            let block = if y == SURFACE_Y {
                Block::GrassBlock
            } else if y >= SURFACE_Y - 3 {
                Block::Dirt
            } else {
                Block::Stone
            } as u16;
            let section = (y - WORLD_MIN_Y) as usize / SECTION_SIZE as usize;
            let local_y = (y - WORLD_MIN_Y) as u8 % SECTION_SIZE as u8;
            for lx in 0..16u8 {
                for lz in 0..16u8 {
                    let index = ChunkSection::index(lx, local_y, lz);
                    self.sections[section].blocks[index] = block;
                }
            }
        }
    }

    #[must_use]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        if ChunkPos::from_world(x, z) != self.pos {
            return None;
        }
        let section = self.section_for_y(y)?;
        Some(self.sections[section].get_local(
            x.rem_euclid(CHUNK_WIDTH) as u8,
            (y - WORLD_MIN_Y) as u8 % SECTION_SIZE as u8,
            z.rem_euclid(CHUNK_WIDTH) as u8,
        ))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) -> Option<Block> {
        if ChunkPos::from_world(x, z) != self.pos {
            return None;
        }
        let section = self.section_for_y(y)?;
        Some(self.sections[section].set_local(
            x.rem_euclid(CHUNK_WIDTH) as u8,
            (y - WORLD_MIN_Y) as u8 % SECTION_SIZE as u8,
            z.rem_euclid(CHUNK_WIDTH) as u8,
            block,
        ))
    }

    #[must_use]
    pub fn section(&self, index: usize) -> Option<&ChunkSection> {
        self.sections.get(index)
    }

    #[must_use]
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

#[derive(Default)]
pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl ChunkManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.chunks
            .entry(pos)
            .or_insert_with(|| Chunk::new(pos.x, pos.z))
    }

    #[must_use]
    pub fn get(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn unload(&mut self, pos: ChunkPos) -> bool {
        self.chunks.remove(&pos).is_some()
    }

    #[must_use]
    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn loaded_positions(&self) -> Vec<ChunkPos> {
        self.chunks.keys().copied().collect()
    }
}

pub struct DroppedItem {
    pub entity: Entity,
    pub stack: ItemStack,
    pub age: u64,
}

pub const DROP_DESPAWN_AGE: u64 = 6000;
pub const DROP_PICKUP_DELAY: u64 = 10;

pub struct World {
    name: String,
    dimension: String,
    min_y: i32,
    height: i32,
    spawn: Spawn,
    chunks: ChunkManager,
    drops: Vec<DroppedItem>,
}

impl World {
    #[must_use]
    pub fn new(name: String, dimension: String) -> Self {
        Self {
            name,
            dimension,
            min_y: WORLD_MIN_Y,
            height: WORLD_HEIGHT,
            spawn: Spawn::default_spawn(),
            chunks: ChunkManager::new(),
            drops: Vec::new(),
        }
    }

    pub fn spawn_drop(&mut self, entity: Entity, stack: ItemStack) {
        self.drops.push(DroppedItem {
            entity,
            stack,
            age: 0,
        });
    }

    #[must_use]
    pub fn drops(&self) -> &[DroppedItem] {
        &self.drops
    }

    pub fn drops_mut(&mut self) -> &mut Vec<DroppedItem> {
        &mut self.drops
    }

    pub fn remove_drop(&mut self, entity_id: i32) -> bool {
        match self
            .drops
            .iter()
            .position(|drop| drop.entity.id == entity_id)
        {
            Some(index) => {
                self.drops.swap_remove(index);
                true
            }
            None => false,
        }
    }

    #[must_use]
    pub fn default_world() -> Self {
        Self::new(
            DEFAULT_WORLD_NAME.to_owned(),
            DEFAULT_DIMENSION_ID.to_owned(),
        )
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    #[must_use]
    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    #[must_use]
    pub fn height(&self) -> i32 {
        self.height
    }

    #[must_use]
    pub fn spawn(&self) -> Spawn {
        self.spawn
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.chunks.load(pos)
    }

    #[must_use]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(pos)
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) -> bool {
        self.chunks.unload(pos)
    }

    #[must_use]
    pub fn is_chunk_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.is_loaded(pos)
    }

    #[must_use]
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.loaded_count()
    }

    #[must_use]
    pub fn ground_top(&self, x: f64, z: f64, from_y: f64) -> Option<f64> {
        if from_y < self.min_y as f64 {
            return None;
        }
        let (bx, bz) = (x.floor() as i32, z.floor() as i32);
        let feet_solid = |y: i32| -> bool {
            matches!(self.get_block(bx, y, bz), Some(block) if block.is_solid())
        };
        let mut y = from_y.floor() as i32;
        if feet_solid(y) || feet_solid(y + 1) {
            let ceiling = self.min_y + self.height + 1;
            while y <= ceiling {
                if !feet_solid(y) && !feet_solid(y + 1) {
                    return Some(y as f64);
                }
                y += 1;
            }
            return None;
        }
        while y >= self.min_y {
            if feet_solid(y) {
                return Some(y as f64 + 1.0);
            }
            y -= 1;
        }
        None
    }

    #[must_use]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        if !(self.min_y..self.min_y + self.height).contains(&y) {
            return None;
        }
        self.chunks
            .get(ChunkPos::from_world(x, z))
            .and_then(|chunk| chunk.get_block(x, y, z))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) -> Option<Block> {
        if !(self.min_y..self.min_y + self.height).contains(&y) {
            return None;
        }
        let pos = ChunkPos::from_world(x, z);
        self.chunks.load(pos);
        self.chunks
            .get_mut(pos)
            .and_then(|chunk| chunk.set_block(x, y, z, block))
    }
}

#[derive(Default)]
pub struct WorldManager {
    worlds: HashMap<String, World>,
}

impl WorldManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn create_default() -> Self {
        let mut manager = Self::new();
        manager.register(World::default_world());
        manager
    }

    pub fn register(&mut self, world: World) -> bool {
        let name = world.name().to_owned();
        if self.worlds.contains_key(&name) {
            return false;
        }
        self.worlds.insert(name, world);
        true
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&World> {
        self.worlds.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut World> {
        self.worlds.get_mut(name)
    }

    pub fn unload(&mut self, name: &str) -> bool {
        self.worlds.remove(name).is_some()
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.worlds.contains_key(name)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.worlds.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.worlds.is_empty()
    }

    pub fn names(&self) -> Vec<String> {
        self.worlds.keys().cloned().collect()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, String, World> {
        self.worlds.values_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_conversion_at_boundaries() {
        for (world, chunk, local) in [
            (0, 0, 0),
            (15, 0, 15),
            (16, 1, 0),
            (17, 1, 1),
            (-1, -1, 15),
            (-16, -1, 0),
            (-17, -2, 15),
        ] {
            assert_eq!(ChunkPos::from_world(world, 0).x, chunk, "x={world}");
            assert_eq!(world.rem_euclid(16) as u8, local, "x={world}");
        }
        for (world, chunk, local) in [(0, 0, 0), (-1, -1, 15), (31, 1, 15), (-33, -3, 15)] {
            assert_eq!(ChunkPos::from_world(0, world).z, chunk, "z={world}");
            assert_eq!(world.rem_euclid(16) as u8, local, "z={world}");
        }
        assert_eq!(
            BlockPos::new(-1, 64, -17).chunk_pos(),
            ChunkPos::new(-1, -2)
        );
        assert_eq!(BlockPos::new(-1, 64, 0).local_x(), 15);
        assert_eq!(BlockPos::new(0, 64, -1).local_z(), 15);
        assert_eq!(ChunkPos::new(2, -3).min_block(), (32, -48));
    }

    #[test]
    fn sections_store_and_report_empty() {
        let mut section = ChunkSection::new(0);
        assert!(section.is_empty());
        assert_eq!(section.section_y(), 0);
        assert_eq!(section.get_local(0, 0, 0), Block::Air);
        assert_eq!(section.set_local(1, 2, 3, Block::Stone), Block::Air);
        assert_eq!(section.get_local(1, 2, 3), Block::Stone);
        assert!(!section.is_empty());
        assert_eq!(section.set_local(1, 2, 3, Block::Air), Block::Stone);
        assert!(section.is_empty());
    }

    #[test]
    fn chunk_blocks_across_sections_and_boundaries() {
        let mut chunk = Chunk::new(0, 0);
        assert_eq!(chunk.pos(), ChunkPos::new(0, 0));
        assert_eq!(chunk.section_count(), SECTION_COUNT);
        assert_eq!(chunk.get_block(0, SURFACE_Y, 0), Some(Block::GrassBlock));
        assert_eq!(chunk.get_block(0, SURFACE_Y - 1, 0), Some(Block::Dirt));
        assert_eq!(chunk.get_block(0, WORLD_MIN_Y, 0), Some(Block::Stone));
        assert_eq!(chunk.get_block(0, SURFACE_Y + 1, 0), Some(Block::Air));
        assert_eq!(chunk.get_block(0, WORLD_MIN_Y - 1, 0), None);
        assert_eq!(chunk.get_block(0, WORLD_MIN_Y + WORLD_HEIGHT, 0), None);
        assert_eq!(chunk.get_block(16, 64, 0), None);

        let prev = chunk.set_block(15, 100, 15, Block::Stone);
        assert_eq!(prev, Some(Block::Air));
        assert_eq!(chunk.get_block(15, 100, 15), Some(Block::Stone));
        let prev = chunk.set_block(15, 100, 15, Block::Dirt);
        assert_eq!(prev, Some(Block::Stone));

        let mut neg = Chunk::new(-2, -1);
        assert_eq!(neg.get_block(-17, SURFACE_Y, -1), Some(Block::GrassBlock));
        assert_eq!(neg.set_block(-32, 70, -16, Block::Dirt), Some(Block::Air));
        assert_eq!(neg.get_block(-32, 70, -16), Some(Block::Dirt));
    }

    #[test]
    fn section_changes_track_and_drain() {
        let mut chunk = Chunk::new(0, 0);
        for index in 0..chunk.section_count() {
            assert!(!chunk.section(index).unwrap().is_dirty());
        }
        assert_eq!(chunk.set_block(0, 100, 0, Block::Stone), Some(Block::Air));
        let section = (100 - WORLD_MIN_Y) as usize / SECTION_SIZE as usize;
        assert!(chunk.section(section).unwrap().is_dirty());
        let taken = chunk.section_mut(section).unwrap().take_changed();
        assert_eq!(taken, vec![(0, (100 - WORLD_MIN_Y) as u8 % 16, 0)]);
        assert!(!chunk.section(section).unwrap().is_dirty());
        assert_eq!(chunk.set_block(0, 100, 0, Block::Stone), Some(Block::Stone));
        assert!(!chunk.section(section).unwrap().is_dirty());
    }

    #[test]
    fn chunk_manager_load_get_unload() {
        let mut manager = ChunkManager::new();
        let pos = ChunkPos::new(3, -2);
        assert!(!manager.is_loaded(pos));
        assert!(manager.get(pos).is_none());
        manager.load(pos);
        assert!(manager.is_loaded(pos));
        assert_eq!(manager.loaded_count(), 1);
        manager.load(pos);
        assert_eq!(manager.loaded_count(), 1);
        assert!(manager.get_mut(pos).is_some());
        assert!(manager.unload(pos));
        assert!(!manager.is_loaded(pos));
        assert!(!manager.unload(pos));
        manager.load(ChunkPos::new(0, 0));
        manager.load(ChunkPos::new(1, 1));
        let mut positions = manager.loaded_positions();
        positions.sort_by_key(|p| (p.x, p.z));
        assert_eq!(positions, vec![ChunkPos::new(0, 0), ChunkPos::new(1, 1)]);
    }

    #[test]
    fn world_blocks_require_loaded_chunks() {
        let mut world = World::default_world();
        assert_eq!(world.name(), "world");
        assert_eq!(world.dimension(), "minecraft:overworld");
        assert_eq!(world.min_y(), WORLD_MIN_Y);
        assert_eq!(world.height(), WORLD_HEIGHT);
        let spawn = world.spawn();
        assert_eq!((spawn.x, spawn.y, spawn.z), (0.0, 100.0, 0.0));

        assert_eq!(world.get_block(0, SURFACE_Y, 0), None);
        world.load_chunk(ChunkPos::new(0, 0));
        assert!(world.is_chunk_loaded(ChunkPos::new(0, 0)));
        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.get_block(0, SURFACE_Y, 0), Some(Block::GrassBlock));
        assert_eq!(world.set_block(0, 90, 0, Block::Stone), Some(Block::Air));
        assert_eq!(world.get_block(0, 90, 0), Some(Block::Stone));
        assert_eq!(world.set_block(0, WORLD_MIN_Y - 1, 0, Block::Stone), None);
        assert!(world.unload_chunk(ChunkPos::new(0, 0)));
        assert_eq!(world.get_block(0, SURFACE_Y, 0), None);
    }

    #[test]
    fn world_manager_registers_and_unloads() {
        let mut manager = WorldManager::create_default();
        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
        assert!(manager.contains("world"));
        assert_eq!(manager.names(), vec!["world".to_owned()]);
        assert!(manager.get("world").is_some());
        assert!(manager.get_mut("world").is_some());

        assert!(!manager.register(World::default_world()));
        assert_eq!(manager.len(), 1);
        assert!(manager.register(World::new(
            "nether".to_owned(),
            "minecraft:the_nether".to_owned()
        )));
        assert_eq!(manager.len(), 2);
        assert!(manager.get("missing").is_none());
        assert!(manager.unload("nether"));
        assert!(!manager.contains("nether"));
        assert!(!manager.unload("nether"));
        assert!(manager.unload("world"));
        assert!(manager.is_empty());
    }

    #[test]
    fn ground_top_finds_flat_surface() {
        let mut world = World::default_world();
        assert_eq!(world.ground_top(0.0, 0.0, 100.0), None);
        world.load_chunk(ChunkPos::new(0, 0));
        world.load_chunk(ChunkPos::new(-1, 0));
        assert_eq!(world.ground_top(0.0, 0.0, 100.0), Some(65.0));
        assert_eq!(world.ground_top(-0.5, 15.9, 65.0), Some(65.0));
        assert_eq!(world.ground_top(0.0, 0.0, 60.0), Some(65.0));
        assert_eq!(world.ground_top(0.0, 0.0, -100.0), None);
        assert!(Block::Stone.is_solid());
        assert!(Block::GrassBlock.is_solid());
        assert!(!Block::Air.is_solid());
    }

    #[test]
    fn drops_spawn_age_and_remove() {
        use crate::entity::Entity;

        let mut world = World::default_world();
        assert!(world.drops().is_empty());
        world.spawn_drop(
            Entity::spawn(11, None, "world".to_owned(), 1.0, 65.0, 2.0),
            ItemStack::new(crate::inventory::Item::Dirt, 3).unwrap(),
        );
        assert_eq!(world.drops().len(), 1);
        assert_eq!(world.drops()[0].stack.count, 3);
        assert_eq!(world.drops()[0].age, 0);
        assert!(!world.remove_drop(99));
        assert!(world.remove_drop(11));
        assert!(world.drops().is_empty());
    }

    #[test]
    fn block_names_and_ids() {
        assert_eq!(Block::Air.name(), "minecraft:air");
        assert_eq!(Block::Stone.name(), "minecraft:stone");
        assert_eq!(Block::Dirt.name(), "minecraft:dirt");
        assert_eq!(Block::GrassBlock.name(), "minecraft:grass_block");
        assert_eq!(Block::from_id(0), Some(Block::Air));
        assert_eq!(Block::from_id(3), Some(Block::GrassBlock));
        assert_eq!(Block::from_id(99), None);
    }
}
