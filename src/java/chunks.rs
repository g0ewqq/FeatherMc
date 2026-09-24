use super::packets::encode_packet;
use super::proto::Writer;
use crate::world::{Block, Chunk, SECTION_COUNT, SURFACE_Y, WORLD_HEIGHT, WORLD_MIN_Y};

pub const CHUNK_RADIUS: i32 = 6;

const AIR_STATE: i32 = 0;
const STONE_STATE: i32 = 1;
// Grass Block states 8 (snowy) / 9 (dry); 9 is the natural default.
const GRASS_STATE: i32 = 9;
const DIRT_STATE: i32 = 10;
const PLAINS_BIOME: i32 = 0;

const LIGHT_SECTION_COUNT: usize = SECTION_COUNT + 2;
const SKY_FULL: u8 = 0xFF;

#[must_use]
pub fn block_to_state(block: Block) -> i32 {
    match block {
        Block::Air => AIR_STATE,
        Block::Stone => STONE_STATE,
        Block::Dirt => DIRT_STATE,
        Block::GrassBlock => GRASS_STATE,
    }
}

fn pack_bits(values: &[u64], bits_per_entry: u32) -> Vec<u64> {
    if bits_per_entry == 0 {
        return Vec::new();
    }
    let entries_per_long = 64 / bits_per_entry as usize;
    let longs = values.len().div_ceil(entries_per_long);
    let mut out = vec![0u64; longs];
    for (i, &value) in values.iter().enumerate() {
        out[i / entries_per_long] |= value << ((i % entries_per_long) as u32 * bits_per_entry);
    }
    out
}

fn bits_needed(count: usize) -> u32 {
    let mut bits = 0u32;
    while (1usize << bits) < count {
        bits += 1;
    }
    bits
}

fn write_heightmaps(body: &mut Writer) {
    let bits = bits_needed((WORLD_HEIGHT + 1) as usize);
    let top = (SURFACE_Y - WORLD_MIN_Y + 1) as u64;
    let longs = pack_bits(&vec![top; 256], bits);
    body.write_varint(2);
    for kind in [1, 4] {
        body.write_varint(kind);
        body.write_varint(longs.len() as i32);
        for long in &longs {
            body.write_u64(*long);
        }
    }
}

fn write_block_container(body: &mut Writer, states: &[u16; 4096]) {
    let mut palette: Vec<i32> = Vec::new();
    let mut indices = vec![0u64; 4096];
    let mut non_air = 0i16;
    for (i, &local) in states.iter().enumerate() {
        let global = block_to_state(Block::from_id(local).unwrap_or(Block::Air));
        if global != AIR_STATE {
            non_air += 1;
        }
        let slot = match palette.iter().position(|&entry| entry == global) {
            Some(slot) => slot,
            None => {
                palette.push(global);
                palette.len() - 1
            }
        };
        indices[i] = slot as u64;
    }
    body.write_i16(non_air);
    body.write_i16(0);
    if palette.len() == 1 {
        body.write_u8(0);
        body.write_varint(palette[0]);
        return;
    }
    let bits = bits_needed(palette.len()).clamp(4, 8);
    body.write_u8(bits as u8);
    body.write_varint(palette.len() as i32);
    for entry in &palette {
        body.write_varint(*entry);
    }
    for long in pack_bits(&indices, bits) {
        body.write_u64(long);
    }
}

fn write_biome_container(body: &mut Writer) {
    body.write_u8(0);
    body.write_varint(PLAINS_BIOME);
}

fn write_light(body: &mut Writer) {
    let full_mask = if LIGHT_SECTION_COUNT >= 64 {
        u64::MAX
    } else {
        (1u64 << LIGHT_SECTION_COUNT) - 1
    };
    body.write_varint(1);
    body.write_u64(full_mask);
    body.write_varint(0);
    body.write_varint(0);
    body.write_varint(1);
    body.write_u64(full_mask);
    body.write_varint(LIGHT_SECTION_COUNT as i32);
    for _ in 0..LIGHT_SECTION_COUNT {
        body.write_varint(2048);
        body.write_bytes(&[SKY_FULL; 2048]);
    }
    body.write_varint(0);
}

#[must_use]
pub fn encode_chunk_data(chunk: &Chunk) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i32(chunk.pos().x);
    body.write_i32(chunk.pos().z);
    write_heightmaps(&mut body);
    let mut data = Writer::new();
    for index in 0..chunk.section_count() {
        if let Some(section) = chunk.section(index) {
            write_block_container(&mut data, section.states());
            write_biome_container(&mut data);
        }
    }
    let data = data.into_bytes();
    body.write_varint(data.len() as i32);
    body.write_bytes(&data);
    body.write_varint(0);
    write_light(&mut body);
    encode_packet(0x2D, &body.into_bytes())
}

#[must_use]
pub fn encode_set_center(x: i32, z: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(x);
    body.write_varint(z);
    encode_packet(0x5E, &body.into_bytes())
}

#[must_use]
pub fn encode_batch_start() -> Vec<u8> {
    encode_packet(0x0C, &[])
}

#[must_use]
pub fn encode_unload_chunk(x: i32, z: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i32(x);
    body.write_i32(z);
    encode_packet(0x25, &body.into_bytes())
}

#[must_use]
pub fn encode_batch_finished(batch_size: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(batch_size);
    encode_packet(0x0B, &body.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::proto::{decode_varint_prefix, Reader};
    use super::*;

    fn decode_frame(bytes: &[u8]) -> (i32, Vec<u8>) {
        let (len, header) = decode_varint_prefix(bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        let id = reader.read_varint().unwrap();
        (id, bytes[header + reader.position()..].to_vec())
    }

    #[test]
    fn maps_blocks_to_vanilla_states() {
        assert_eq!(block_to_state(Block::Air), 0);
        assert_eq!(block_to_state(Block::Stone), 1);
        assert_eq!(block_to_state(Block::GrassBlock), 9); // dry (default)
        assert_eq!(block_to_state(Block::Dirt), 10);
    }

    #[test]
    fn packs_bits_lsb_first_with_padding() {
        assert!(pack_bits(&[], 4).is_empty());
        assert!(pack_bits(&[1, 2], 0).is_empty());
        let longs = pack_bits(&[1u64, 2, 3], 4);
        assert_eq!(longs.len(), 1);
        assert_eq!(longs[0], 0x321);
        let longs = pack_bits(&vec![0u64; 256], 9);
        assert_eq!(longs.len(), 37);
    }

    #[test]
    fn chunk_packet_has_expected_shape() {
        let mut chunk = Chunk::new(3, -2);
        chunk.set_block(3, 100, -2, Block::Dirt);
        let bytes = encode_chunk_data(&chunk);
        let (id, payload) = decode_frame(&bytes);
        assert_eq!(id, 0x2D);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_i32().unwrap(), 3);
        assert_eq!(reader.read_i32().unwrap(), -2);

        assert_eq!(reader.read_varint().unwrap(), 2);
        for want in [1, 4] {
            assert_eq!(reader.read_varint().unwrap(), want);
            assert_eq!(reader.read_varint().unwrap(), 37);
            for _ in 0..37 {
                reader.read_u64().unwrap();
            }
        }

        let size = reader.read_varint().unwrap() as usize;
        let data = reader.read_bytes(size).unwrap();
        let mut sections = Reader::new(data);
        for _ in 0..SECTION_COUNT {
            let non_air = sections.read_i16().unwrap();
            assert!(non_air >= 0);
            sections.read_i16().unwrap();
            let bits = sections.read_u8().unwrap();
            if bits == 0 {
                sections.read_varint().unwrap();
            } else {
                assert!((4..=8).contains(&bits));
                let len = sections.read_varint().unwrap() as usize;
                assert!((2..=16).contains(&len));
                for _ in 0..len {
                    sections.read_varint().unwrap();
                }
                let entries_per_long = 64 / bits as usize;
                let longs = 4096usize.div_ceil(entries_per_long);
                for _ in 0..longs {
                    sections.read_u64().unwrap();
                }
            }
            assert_eq!(sections.read_u8().unwrap(), 0);
            assert_eq!(sections.read_varint().unwrap(), PLAINS_BIOME);
        }
        assert_eq!(sections.remaining(), 0);

        assert_eq!(reader.read_varint().unwrap(), 0);

        assert_eq!(reader.read_varint().unwrap(), 1);
        reader.read_u64().unwrap();
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 1);
        reader.read_u64().unwrap();
        assert_eq!(reader.read_varint().unwrap(), LIGHT_SECTION_COUNT as i32);
        for _ in 0..LIGHT_SECTION_COUNT {
            assert_eq!(reader.read_varint().unwrap(), 2048);
            assert_eq!(reader.read_bytes(2048).unwrap().len(), 2048);
        }
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn center_and_batch_packets_encode() {
        let (id, payload) = decode_frame(&encode_set_center(5, -7));
        assert_eq!(id, 0x5E);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 5);
        assert_eq!(reader.read_varint().unwrap(), -7);

        let (id, payload) = decode_frame(&encode_batch_start());
        assert_eq!(id, 0x0C);
        assert!(payload.is_empty());

        let (id, payload) = decode_frame(&encode_unload_chunk(-3, 7));
        assert_eq!(id, 0x25);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_i32().unwrap(), -3);
        assert_eq!(reader.read_i32().unwrap(), 7);
        assert_eq!(reader.remaining(), 0);

        let (id, payload) = decode_frame(&encode_batch_finished(81));
        assert_eq!(id, 0x0B);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 81);
    }
}
