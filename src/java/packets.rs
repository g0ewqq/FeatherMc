use super::error::ProtoError;
use super::nbt::NbtTag;
use super::player::PlayerSession;
use super::proto::{Reader, Writer};
use crate::inventory::{ItemStack, INVENTORY_SIZE};

pub const DUPLICATE_LOGIN_REASON: &str = "You logged in from another location.";

const VIEW_DISTANCE: i32 = 10;
const SIMULATION_DISTANCE: i32 = 10;
const SEA_LEVEL: i32 = 63;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginStart {
    pub name: String,
    pub uuid: Option<u128>,
}

pub fn decode_handshake(payload: &[u8]) -> Result<Handshake, ProtoError> {
    let mut reader = Reader::new(payload);
    Ok(Handshake {
        protocol_version: reader.read_varint()?,
        server_address: reader.read_string()?,
        server_port: reader.read_u16()?,
        next_state: reader.read_varint()?,
    })
}

pub fn decode_login_start(payload: &[u8]) -> Result<LoginStart, ProtoError> {
    let mut reader = Reader::new(payload);
    let name = reader.read_string()?;
    let uuid = if reader.remaining() >= 16 {
        Some(reader.read_uuid()?)
    } else {
        None
    };
    Ok(LoginStart { name, uuid })
}

pub fn encode_packet(id: i32, body: &[u8]) -> Vec<u8> {
    let mut id_writer = Writer::new();
    id_writer.write_varint(id);
    let id_bytes = id_writer.into_bytes();

    let mut out = Writer::new();
    out.write_varint(id_bytes.len() as i32 + body.len() as i32);
    out.write_bytes(&id_bytes);
    out.write_bytes(body);
    out.into_bytes()
}

fn escape_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[must_use]
pub fn chat_json(text: &str) -> String {
    format!("{{\"text\":\"{}\"}}", escape_json(text))
}

#[must_use]
pub fn status_json(version_name: &str, protocol: i32, motd: &str, max_players: u32) -> String {
    format!(
        "{{\"version\":{{\"name\":\"{}\",\"protocol\":{}}},\"players\":{{\"max\":{},\"online\":0}},\"description\":{{\"text\":\"{}\"}}}}",
        escape_json(version_name),
        protocol,
        max_players,
        escape_json(motd)
    )
}

pub fn encode_status_response(
    version_name: &str,
    protocol: i32,
    motd: &str,
    max_players: u32,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_string(&status_json(version_name, protocol, motd, max_players));
    encode_packet(0x00, &body.into_bytes())
}

pub fn encode_pong(payload: i64) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i64(payload);
    encode_packet(0x01, &body.into_bytes())
}

pub fn encode_login_success(uuid: u128, name: &str, session_id: u128) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_uuid(uuid);
    body.write_string(name);
    body.write_varint(0); // properties count
    body.write_uuid(session_id);
    encode_packet(0x02, &body.into_bytes())
}

pub fn encode_play_login(
    player: &PlayerSession,
    max_players: u32,
    dimension_type_id: i32,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i32(player.entity_id);
    body.write_bool(false);
    body.write_varint(1);
    body.write_string(&player.dimension);
    body.write_varint(max_players as i32);
    body.write_varint(VIEW_DISTANCE);
    body.write_varint(SIMULATION_DISTANCE);
    body.write_bool(false);
    body.write_bool(true);
    body.write_bool(false);
    body.write_varint(dimension_type_id);
    body.write_string(&player.dimension);
    body.write_i64(0);
    body.write_u8(player.gamemode as u8);
    body.write_i8(-1);
    body.write_bool(false);
    body.write_bool(false);
    body.write_bool(false);
    body.write_varint(0);
    body.write_varint(SEA_LEVEL);
    body.write_bool(false); // online mode
    body.write_bool(false); // enforces secure chat
    encode_packet(0x31, &body.into_bytes())
}

pub fn encode_default_spawn(
    dimension: &str,
    x: i32,
    y: i32,
    z: i32,
    yaw: f32,
    pitch: f32,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_string(dimension);
    body.write_position(x, y, z);
    body.write_f32(yaw);
    body.write_f32(pitch);
    encode_packet(0x61, &body.into_bytes())
}

pub fn encode_sync_position(
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    teleport_id: i32,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(teleport_id);
    body.write_f64(x);
    body.write_f64(y);
    body.write_f64(z);
    body.write_f64(0.0); // velocity x
    body.write_f64(0.0); // velocity y
    body.write_f64(0.0); // velocity z
    body.write_f32(yaw);
    body.write_f32(pitch);
    body.write_i32(0); // flags (Teleport Flags, 4-byte int)
    encode_packet(0x48, &body.into_bytes())
}

pub fn encode_game_event(event: u8, value: f32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_u8(event);
    body.write_f32(value);
    encode_packet(0x26, &body.into_bytes())
}

/// Ability flag bits shared by the clientbound and serverbound
/// Player Abilities packets.
pub const ABILITY_INVULNERABLE: u8 = 0x01;
pub const ABILITY_FLYING: u8 = 0x02;
pub const ABILITY_ALLOW_FLYING: u8 = 0x04;
pub const ABILITY_INSTANT_BUILD: u8 = 0x08;

/// Sent on play entry for creative players and whenever flight flips; tells
/// the client which abilities are live.
pub fn encode_player_abilities(flags: u8) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_u8(flags);
    body.write_f32(0.05); // flying speed
    body.write_f32(0.1); // field of view modifier
    encode_packet(0x40, &body.into_bytes())
}

/// Overlay-free system chat line. The content is a plain-text component
/// sent as an NBT string tag, which vanilla renders directly in chat.
pub fn encode_system_chat(text: &str) -> Vec<u8> {
    let mut body = Writer::new();
    if NbtTag::String(text.to_owned())
        .encode_unnamed(&mut body)
        .is_err()
    {
        return Vec::new();
    }
    body.write_bool(false);
    encode_packet(0x79, &body.into_bytes())
}

/// Syncs hearts (and stubbed food) to the client. Health at or below zero
/// shows the death screen.
pub fn encode_health(health: f32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_f32(health);
    body.write_varint(20); // food: hunger is not simulated
    body.write_f32(5.0); // saturation
    encode_packet(0x68, &body.into_bytes())
}

/// Re-spawns a dead player. Mirrors the dimension fields of play login plus
/// the respawn-only tail; data-kept is zero (normal death keeps no
/// client-side data; the server separately keeps the inventory).
pub fn encode_respawn(player: &PlayerSession, dimension_type_id: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(dimension_type_id);
    body.write_string(&player.dimension);
    body.write_i64(0); // hashed seed
    body.write_u8(player.gamemode as u8);
    body.write_i8(-1); // no previous game mode
    body.write_bool(false); // debug world
    body.write_bool(true); // flat world
    body.write_bool(false); // no death location
    body.write_varint(0); // portal cooldown
    body.write_varint(SEA_LEVEL);
    body.write_u8(0); // data kept: none
    encode_packet(0x52, &body.into_bytes())
}

pub fn encode_block_update(x: i32, y: i32, z: i32, state: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_position(x, y, z);
    body.write_varint(state);
    encode_packet(0x08, &body.into_bytes())
}

pub fn write_slot(writer: &mut Writer, stack: ItemStack) {
    if stack.is_empty() {
        writer.write_varint(0);
        return;
    }
    writer.write_varint(stack.count as i32);
    writer.write_varint(stack.item.id());
    writer.write_varint(0);
    writer.write_varint(0);
}

pub fn encode_container_content(
    slots: &[ItemStack; INVENTORY_SIZE],
    cursor: ItemStack,
    state: i32,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(0);
    body.write_varint(state);
    body.write_varint(INVENTORY_SIZE as i32);
    for slot in slots.iter() {
        write_slot(&mut body, *slot);
    }
    write_slot(&mut body, cursor);
    encode_packet(0x12, &body.into_bytes())
}

pub fn encode_container_slot(window: i32, state: i32, index: i32, stack: ItemStack) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(window);
    body.write_varint(state);
    body.write_i16(index as i16);
    write_slot(&mut body, stack);
    encode_packet(0x14, &body.into_bytes())
}

pub fn encode_held_slot(index: u8) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(i32::from(index));
    encode_packet(0x69, &body.into_bytes())
}

pub fn encode_block_changed_ack(sequence: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(sequence);
    encode_packet(0x04, &body.into_bytes())
}

pub fn encode_keep_alive(id: i64) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i64(id);
    encode_packet(0x2C, &body.into_bytes())
}

pub fn encode_known_packs() -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(1);
    body.write_string("minecraft");
    body.write_string("core");
    body.write_string(crate::java::VERSION_NAME);
    encode_packet(0x0E, &body.into_bytes())
}

/// Item tags the client needs to resolve vanilla sulfur archetype NBT.
///
/// Every `sulfur_cube_archetype` entry's `items` field points at one of these.
/// We send them empty — the tag only has to exist for the lookup to succeed,
/// no numeric item ids required.
pub const SULFUR_ITEM_TAGS: [&str; 14] = [
    "minecraft:sulfur_cube_archetype/regular",
    "minecraft:sulfur_cube_archetype/bouncy",
    "minecraft:sulfur_cube_archetype/slow_bouncy",
    "minecraft:sulfur_cube_archetype/slow_flat",
    "minecraft:sulfur_cube_archetype/fast_flat",
    "minecraft:sulfur_cube_archetype/light",
    "minecraft:sulfur_cube_archetype/fast_sliding",
    "minecraft:sulfur_cube_archetype/slow_sliding",
    "minecraft:sulfur_cube_archetype/high_resistance",
    "minecraft:sulfur_cube_archetype/sticky",
    "minecraft:sulfur_cube_archetype/explosive",
    "minecraft:sulfur_cube_archetype/hot",
    "minecraft:sulfur_cube_food",
    "minecraft:sulfur_cube_swallowable",
];

/// Damage-type tags referenced by default item components (fireResistant, etc).
pub const DAMAGE_TYPE_TAGS: [&str; 3] = [
    "minecraft:bypasses_shield",
    "minecraft:is_explosion",
    "minecraft:is_fire",
];

/// Banner-pattern tags referenced by default item components.
pub const BANNER_PATTERN_TAGS: [&str; 10] = [
    "minecraft:pattern_item/bordure_indented",
    "minecraft:pattern_item/creeper",
    "minecraft:pattern_item/field_masoned",
    "minecraft:pattern_item/flow",
    "minecraft:pattern_item/flower",
    "minecraft:pattern_item/globe",
    "minecraft:pattern_item/guster",
    "minecraft:pattern_item/mojang",
    "minecraft:pattern_item/piglin",
    "minecraft:pattern_item/skull",
];

fn write_empty_tags(body: &mut Writer, registry: &str, tags: &[&str]) {
    body.write_string(registry);
    body.write_varint(tags.len() as i32);
    for tag in tags {
        body.write_string(tag);
        body.write_varint(0); // zero entries
    }
}

pub fn encode_update_tags() -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(3); // item, damage_type, banner_pattern
    write_empty_tags(&mut body, "minecraft:item", &SULFUR_ITEM_TAGS);
    write_empty_tags(&mut body, "minecraft:damage_type", &DAMAGE_TYPE_TAGS);
    write_empty_tags(&mut body, "minecraft:banner_pattern", &BANNER_PATTERN_TAGS);
    encode_packet(0x0D, &body.into_bytes())
}

pub fn encode_finish_configuration() -> Vec<u8> {
    encode_packet(0x03, &[])
}

pub fn encode_login_disconnect(reason: &str) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_string(&chat_json(reason));
    encode_packet(0x00, &body.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::proto::decode_varint_prefix;
    use super::*;

    fn handshake_bytes(protocol: i32, address: &str, port: u16, next: i32) -> Vec<u8> {
        let mut body = Writer::new();
        body.write_varint(protocol);
        body.write_string(address);
        body.write_u16(port);
        body.write_varint(next);
        let body = body.into_bytes();
        encode_packet(0x00, &body)
    }

    #[test]
    fn decodes_handshake() {
        let bytes = handshake_bytes(776, "localhost", 25565, 1);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let payload = &bytes[header..header + len as usize];
        let mut reader = Reader::new(payload);
        let id = reader.read_varint().unwrap();
        assert_eq!(id, 0x00);
        let handshake = decode_handshake(&payload[1..]).unwrap();
        assert_eq!(
            handshake,
            Handshake {
                protocol_version: 776,
                server_address: "localhost".to_owned(),
                server_port: 25565,
                next_state: 1,
            }
        );
    }

    #[test]
    fn handshake_rejects_truncated_input() {
        let bytes = handshake_bytes(776, "localhost", 25565, 1);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let payload = &bytes[header + 1..header + len as usize];
        assert_eq!(
            decode_handshake(&payload[..payload.len() - 2]),
            Err(ProtoError::Truncated)
        );
    }

    #[test]
    fn decodes_login_start_with_and_without_uuid() {
        let mut body = Writer::new();
        body.write_string("TestPlayer");
        body.write_uuid(0x123E_4567_E89B_12D3_A456_4266_1417_4000);
        assert_eq!(
            decode_login_start(&body.into_bytes()).unwrap(),
            LoginStart {
                name: "TestPlayer".to_owned(),
                uuid: Some(0x123E_4567_E89B_12D3_A456_4266_1417_4000),
            }
        );

        let mut body = Writer::new();
        body.write_string("OldClient");
        assert_eq!(
            decode_login_start(&body.into_bytes()).unwrap(),
            LoginStart {
                name: "OldClient".to_owned(),
                uuid: None,
            }
        );
    }

    #[test]
    fn status_json_has_vanilla_shape() {
        let json = status_json("26.2", 776, "A SpironMC Server", 20);
        assert_eq!(
            json,
            "{\"version\":{\"name\":\"26.2\",\"protocol\":776},\"players\":{\"max\":20,\"online\":0},\"description\":{\"text\":\"A SpironMC Server\"}}"
        );
    }

    #[test]
    fn json_escapes_special_characters() {
        assert_eq!(chat_json("a\"b\\c"), "{\"text\":\"a\\\"b\\\\c\"}");
        assert_eq!(
            status_json("v", 1, "a\nb\ttab", 5),
            "{\"version\":{\"name\":\"v\",\"protocol\":1},\"players\":{\"max\":5,\"online\":0},\"description\":{\"text\":\"a\\nb\\ttab\"}}"
        );
    }

    #[test]
    fn encoded_packets_have_length_prefix_and_id() {
        let bytes = encode_pong(123456789);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        assert_eq!(len as usize, bytes.len() - header);
        let mut reader = Reader::new(&bytes[header..]);
        assert_eq!(reader.read_varint().unwrap(), 0x01);
        assert_eq!(reader.read_i64().unwrap(), 123456789);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn login_success_carries_profile() {
        let bytes = encode_login_success(
            0x123E_4567_E89B_12D3_A456_4266_1417_4000,
            "Steve",
            0xAAAAAAAA_BBBBBBBB_CCCCCCCC_DDDDDDDD,
        );
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x02);
        assert_eq!(
            reader.read_uuid().unwrap(),
            0x123E_4567_E89B_12D3_A456_4266_1417_4000
        );
        assert_eq!(reader.read_string().unwrap(), "Steve");
        assert_eq!(reader.read_varint().unwrap(), 0); // properties count
        assert_eq!(
            reader.read_uuid().unwrap(),
            0xAAAAAAAA_BBBBBBBB_CCCCCCCC_DDDDDDDD
        );
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn configuration_packets_encode() {
        let bytes = encode_known_packs();
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x0E);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_string().unwrap(), "minecraft");
        assert_eq!(reader.read_string().unwrap(), "core");
        assert_eq!(reader.read_string().unwrap(), "26.2");
        assert_eq!(reader.remaining(), 0);

        let bytes = encode_finish_configuration();
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        assert_eq!(len, 1);
        assert_eq!(bytes[header], 0x03);

        let bytes = encode_update_tags();
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x0D);
        assert_eq!(reader.read_varint().unwrap(), 3);
        for (registry, tags) in [
            ("minecraft:item", SULFUR_ITEM_TAGS.as_slice()),
            ("minecraft:damage_type", DAMAGE_TYPE_TAGS.as_slice()),
            ("minecraft:banner_pattern", BANNER_PATTERN_TAGS.as_slice()),
        ] {
            assert_eq!(reader.read_string().unwrap(), registry);
            assert_eq!(reader.read_varint().unwrap() as usize, tags.len());
            for tag in tags {
                assert_eq!(reader.read_string().unwrap(), *tag);
                assert_eq!(reader.read_varint().unwrap(), 0);
            }
        }
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn block_packets_encode() {
        let bytes = encode_block_update(3, 64, -2, 1);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x08);
        assert_eq!(reader.read_position().unwrap(), (3, 64, -2));
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.remaining(), 0);

        let bytes = encode_block_changed_ack(41);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x04);
        assert_eq!(reader.read_varint().unwrap(), 41);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn slots_encode_empty_and_full() {
        let mut writer = Writer::new();
        write_slot(&mut writer, ItemStack::empty());
        assert_eq!(writer.into_bytes(), vec![0x00]);

        let mut writer = Writer::new();
        write_slot(
            &mut writer,
            ItemStack::new(crate::inventory::Item::Dirt, 5).unwrap(),
        );
        let bytes = writer.into_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_varint().unwrap(), 5);
        assert_eq!(
            reader.read_varint().unwrap(),
            crate::inventory::Item::Dirt.id()
        );
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn container_and_held_packets_encode() {
        let mut slots = [ItemStack::empty(); crate::inventory::INVENTORY_SIZE];
        slots[36] = ItemStack::new(crate::inventory::Item::Stone, 64).unwrap();
        let cursor = ItemStack::new(crate::inventory::Item::Dirt, 7).unwrap();
        let bytes = encode_container_content(&slots, cursor, 3);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x12);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 3);
        assert_eq!(
            reader.read_varint().unwrap(),
            crate::inventory::INVENTORY_SIZE as i32
        );
        for index in 0..crate::inventory::INVENTORY_SIZE {
            let count = reader.read_varint().unwrap();
            if index == 36 {
                assert_eq!(count, 64);
                assert_eq!(
                    reader.read_varint().unwrap(),
                    crate::inventory::Item::Stone.id()
                );
                assert_eq!(reader.read_varint().unwrap(), 0);
                assert_eq!(reader.read_varint().unwrap(), 0);
            } else {
                assert_eq!(count, 0);
            }
        }
        assert_eq!(reader.read_varint().unwrap(), 7);
        assert_eq!(
            reader.read_varint().unwrap(),
            crate::inventory::Item::Dirt.id()
        );
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);

        let bytes = encode_container_slot(0, 4, 37, slots[36]);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x14);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 4);
        assert_eq!(reader.read_i16().unwrap(), 37);
        assert_eq!(reader.read_varint().unwrap(), 64);
        assert_eq!(
            reader.read_varint().unwrap(),
            crate::inventory::Item::Stone.id()
        );
        assert_eq!(reader.remaining(), 2);

        let bytes = encode_held_slot(8);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x69);
        assert_eq!(reader.read_varint().unwrap(), 8);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn game_event_start_chunks_encodes() {
        let bytes = encode_game_event(13, 0.0);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x26);
        assert_eq!(reader.read_u8().unwrap(), 13);
        assert_eq!(reader.read_f32().unwrap(), 0.0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn play_login_carries_required_fields() {
        let player = PlayerSession::new(1, "Steve".to_owned(), Some(99), 776, 7, 8);
        let bytes = encode_play_login(&player, 20, 0);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x31);
        assert_eq!(reader.read_i32().unwrap(), 7);
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_varint().unwrap(), 20);
        assert_eq!(reader.read_varint().unwrap(), 10);
        assert_eq!(reader.read_varint().unwrap(), 10);
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_i64().unwrap(), 0);
        assert_eq!(reader.read_u8().unwrap(), 0);
        assert_eq!(reader.read_i8().unwrap(), -1);
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 63); // sea level
        assert!(!reader.read_bool().unwrap()); // online mode
        assert!(!reader.read_bool().unwrap()); // enforces secure chat
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn spawn_and_sync_encode() {
        let bytes = encode_default_spawn("minecraft:overworld", 0, 100, 0, 0.0, 0.0);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x61);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_position().unwrap(), (0, 100, 0));
        assert_eq!(reader.read_f32().unwrap(), 0.0); // yaw
        assert_eq!(reader.read_f32().unwrap(), 0.0); // pitch
        assert_eq!(reader.remaining(), 0);

        let bytes = encode_sync_position(0.5, 100.0, -3.25, 90.0, -5.0, 12);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x48);
        assert_eq!(reader.read_varint().unwrap(), 12);
        assert_eq!(reader.read_f64().unwrap(), 0.5);
        assert_eq!(reader.read_f64().unwrap(), 100.0);
        assert_eq!(reader.read_f64().unwrap(), -3.25);
        assert_eq!(reader.read_f64().unwrap(), 0.0); // velocity x
        assert_eq!(reader.read_f64().unwrap(), 0.0); // velocity y
        assert_eq!(reader.read_f64().unwrap(), 0.0); // velocity z
        assert_eq!(reader.read_f32().unwrap(), 90.0);
        assert_eq!(reader.read_f32().unwrap(), -5.0);
        assert_eq!(reader.read_i32().unwrap(), 0); // flags
        assert_eq!(reader.remaining(), 0);

        let bytes = encode_keep_alive(987654321);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x2C);
        assert_eq!(reader.read_i64().unwrap(), 987654321);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn player_abilities_encodes_flags() {
        let bytes = encode_player_abilities(0x0E);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x40);
        assert_eq!(reader.read_u8().unwrap(), 0x0E);
        assert_eq!(reader.read_f32().unwrap(), 0.05); // flying speed
        assert_eq!(reader.read_f32().unwrap(), 0.1); // field of view modifier
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn system_chat_encodes_plain_text_without_overlay() {
        let bytes = encode_system_chat("Hello there");
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x79);
        let tag = crate::java::nbt::NbtTag::decode_unnamed(&mut reader).unwrap();
        assert_eq!(
            tag,
            crate::java::nbt::NbtTag::String("Hello there".to_owned())
        );
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn health_and_respawn_encode() {
        let bytes = encode_health(7.5);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x68);
        assert_eq!(reader.read_f32().unwrap(), 7.5);
        assert_eq!(reader.read_varint().unwrap(), 20);
        assert_eq!(reader.read_f32().unwrap(), 5.0);
        assert_eq!(reader.remaining(), 0);

        let player = PlayerSession::new(1, "Steve".to_owned(), None, 776, 3, 9999);
        let bytes = encode_respawn(&player, 0);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x52);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_i64().unwrap(), 0);
        assert_eq!(reader.read_u8().unwrap(), 0); // survival
        assert_eq!(reader.read_i8().unwrap(), -1);
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap()); // flat
        assert!(!reader.read_bool().unwrap()); // no death location
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 63);
        assert_eq!(reader.read_u8().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn login_disconnect_carries_reason() {
        let bytes = encode_login_disconnect(DUPLICATE_LOGIN_REASON);
        let (len, header) = decode_varint_prefix(&bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        assert_eq!(reader.read_varint().unwrap(), 0x00);
        let reason = reader.read_string().unwrap();
        assert!(reason.contains(DUPLICATE_LOGIN_REASON));
    }
}
