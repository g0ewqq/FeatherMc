use super::metadata::{encode_metadata, MetadataEntry};
use super::packets::encode_packet;
use super::proto::Writer;

pub const PLAYER_ENTITY_TYPE: i32 = 156;

const INFO_ADD_PLAYER: u8 = 0x01;
const INFO_UPDATE_GAMEMODE: u8 = 0x04;
const INFO_UPDATE_LISTED: u8 = 0x08;
const INFO_UPDATE_LATENCY: u8 = 0x10;

#[must_use]
pub fn angle_to_byte(angle: f32) -> u8 {
    let wrapped = ((angle % 360.0) + 360.0) % 360.0;
    (wrapped / 360.0 * 256.0) as u8
}

#[must_use]
pub fn pitch_to_byte(pitch: f32) -> u8 {
    (pitch.clamp(-90.0, 90.0) / 360.0 * 256.0) as u8
}

#[must_use]
pub fn encode_player_info_add(uuid: u128, name: &str, gamemode: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_u8(
        INFO_ADD_PLAYER | INFO_UPDATE_GAMEMODE | INFO_UPDATE_LISTED | INFO_UPDATE_LATENCY,
    );
    body.write_varint(1);
    body.write_uuid(uuid);
    body.write_string(name);
    body.write_varint(0);
    body.write_varint(gamemode);
    body.write_bool(true);
    body.write_varint(0);
    encode_packet(0x46, &body.into_bytes())
}

#[must_use]
pub fn encode_player_info_remove(uuids: &[u128]) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(uuids.len() as i32);
    for uuid in uuids {
        body.write_uuid(*uuid);
    }
    encode_packet(0x45, &body.into_bytes())
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn encode_spawn_player(
    entity_id: i32,
    uuid: u128,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    body.write_uuid(uuid);
    body.write_varint(PLAYER_ENTITY_TYPE);
    body.write_f64(x);
    body.write_f64(y);
    body.write_f64(z);
    body.write_varint(0);
    body.write_varint(0);
    body.write_varint(0);
    body.write_u8(pitch_to_byte(pitch));
    body.write_u8(angle_to_byte(yaw));
    body.write_u8(angle_to_byte(yaw));
    body.write_varint(0);
    encode_packet(0x01, &body.into_bytes())
}

#[must_use]
pub fn encode_entity_metadata(entity_id: i32, entries: &[MetadataEntry]) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    encode_metadata(entries, &mut body);
    encode_packet(0x63, &body.into_bytes())
}

#[must_use]
pub fn encode_entity_position(
    entity_id: i32,
    dx: i16,
    dy: i16,
    dz: i16,
    on_ground: bool,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    body.write_i16(dx);
    body.write_i16(dy);
    body.write_i16(dz);
    body.write_bool(on_ground);
    encode_packet(0x35, &body.into_bytes())
}

#[must_use]
pub fn encode_entity_position_rotation(
    entity_id: i32,
    dx: i16,
    dy: i16,
    dz: i16,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    body.write_i16(dx);
    body.write_i16(dy);
    body.write_i16(dz);
    body.write_u8(angle_to_byte(yaw));
    body.write_u8(pitch_to_byte(pitch));
    body.write_bool(on_ground);
    encode_packet(0x36, &body.into_bytes())
}

#[must_use]
pub fn encode_entity_rotation(entity_id: i32, yaw: f32, pitch: f32, on_ground: bool) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    body.write_u8(angle_to_byte(yaw));
    body.write_u8(pitch_to_byte(pitch));
    body.write_bool(on_ground);
    encode_packet(0x38, &body.into_bytes())
}

#[must_use]
pub fn encode_head_rotation(entity_id: i32, head_yaw: f32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    body.write_u8(angle_to_byte(head_yaw));
    encode_packet(0x53, &body.into_bytes())
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn encode_teleport_entity(
    entity_id: i32,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_id);
    body.write_f64(x);
    body.write_f64(y);
    body.write_f64(z);
    body.write_f64(0.0);
    body.write_f64(0.0);
    body.write_f64(0.0);
    body.write_f32(yaw);
    body.write_f32(pitch);
    body.write_i32(0);
    body.write_bool(on_ground);
    encode_packet(0x7D, &body.into_bytes())
}

#[must_use]
pub fn encode_destroy_entities(entity_ids: &[i32]) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(entity_ids.len() as i32);
    for id in entity_ids {
        body.write_varint(*id);
    }
    encode_packet(0x4D, &body.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::metadata::{MetadataKind, FLAG_SNEAKING, POSE_CROUCHING};
    use super::super::proto::{decode_varint_prefix, Reader};
    use super::*;

    fn decode_frame(bytes: &[u8]) -> (i32, Vec<u8>) {
        let (len, header) = decode_varint_prefix(bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        let id = reader.read_varint().unwrap();
        (id, bytes[header + reader.position()..].to_vec())
    }

    #[test]
    fn info_add_carries_profile() {
        let bytes = encode_player_info_add(0x99, "Steve", 0);
        let (id, payload) = decode_frame(&bytes);
        assert_eq!(id, 0x46);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_u8().unwrap(), 0x1D);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_uuid().unwrap(), 0x99);
        assert_eq!(reader.read_string().unwrap(), "Steve");
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert!(reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn info_remove_lists_uuids() {
        let bytes = encode_player_info_remove(&[1, 2]);
        let (id, payload) = decode_frame(&bytes);
        assert_eq!(id, 0x45);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 2);
        assert_eq!(reader.read_uuid().unwrap(), 1);
        assert_eq!(reader.read_uuid().unwrap(), 2);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn spawn_player_has_type_and_pose() {
        let bytes = encode_spawn_player(7, 0xABCDEF, 1.5, 65.0, -3.25, 90.0, -10.0);
        let (id, payload) = decode_frame(&bytes);
        assert_eq!(id, 0x01);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 7);
        assert_eq!(reader.read_uuid().unwrap(), 0xABCDEF);
        assert_eq!(reader.read_varint().unwrap(), PLAYER_ENTITY_TYPE);
        assert_eq!(reader.read_f64().unwrap(), 1.5);
        assert_eq!(reader.read_f64().unwrap(), 65.0);
        assert_eq!(reader.read_f64().unwrap(), -3.25);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_u8().unwrap(), pitch_to_byte(-10.0));
        assert_eq!(reader.read_u8().unwrap(), angle_to_byte(90.0));
        assert_eq!(reader.read_u8().unwrap(), angle_to_byte(90.0));
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn movement_packets_carry_deltas_and_angles() {
        let (id, payload) = decode_frame(&encode_entity_position(3, 10, -20, 30, true));
        assert_eq!(id, 0x35);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 3);
        assert_eq!(reader.read_i16().unwrap(), 10);
        assert_eq!(reader.read_i16().unwrap(), -20);
        assert_eq!(reader.read_i16().unwrap(), 30);
        assert!(reader.read_bool().unwrap());

        let (id, payload) = decode_frame(&encode_entity_position_rotation(
            3, 1, 2, 3, 45.0, 0.0, false,
        ));
        assert_eq!(id, 0x36);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 3);
        assert_eq!(reader.read_i16().unwrap(), 1);
        assert_eq!(reader.read_i16().unwrap(), 2);
        assert_eq!(reader.read_i16().unwrap(), 3);
        assert_eq!(reader.read_u8().unwrap(), angle_to_byte(45.0));
        assert_eq!(reader.read_u8().unwrap(), pitch_to_byte(0.0));
        assert!(!reader.read_bool().unwrap());

        let (id, payload) = decode_frame(&encode_entity_rotation(3, 180.0, 30.0, true));
        assert_eq!(id, 0x38);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 3);
        assert_eq!(reader.read_u8().unwrap(), angle_to_byte(180.0));

        let (id, payload) = decode_frame(&encode_head_rotation(3, 270.0));
        assert_eq!(id, 0x53);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 3);
        assert_eq!(reader.read_u8().unwrap(), angle_to_byte(270.0));

        let (id, payload) =
            decode_frame(&encode_teleport_entity(3, 1.0, 2.0, 3.0, 10.0, 20.0, true));
        assert_eq!(id, 0x7D);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 3);
        assert_eq!(reader.read_f64().unwrap(), 1.0);
        assert_eq!(reader.read_f64().unwrap(), 2.0);
        assert_eq!(reader.read_f64().unwrap(), 3.0);
        assert_eq!(reader.read_f64().unwrap(), 0.0);
        assert_eq!(reader.read_f64().unwrap(), 0.0);
        assert_eq!(reader.read_f64().unwrap(), 0.0);
        assert_eq!(reader.read_f32().unwrap(), 10.0);
        assert_eq!(reader.read_f32().unwrap(), 20.0);
        assert_eq!(reader.read_i32().unwrap(), 0);
        assert!(reader.read_bool().unwrap());

        let (id, payload) = decode_frame(&encode_destroy_entities(&[4, 9]));
        assert_eq!(id, 0x4D);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 2);
        assert_eq!(reader.read_varint().unwrap(), 4);
        assert_eq!(reader.read_varint().unwrap(), 9);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn metadata_packet_wraps_entries() {
        let entries = vec![
            MetadataEntry {
                index: 0,
                kind: MetadataKind::Byte(FLAG_SNEAKING),
            },
            MetadataEntry {
                index: 6,
                kind: MetadataKind::Pose(POSE_CROUCHING),
            },
        ];
        let (id, payload) = decode_frame(&encode_entity_metadata(11, &entries));
        assert_eq!(id, 0x63);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 11);
        let back = super::super::metadata::decode_metadata(&mut reader).unwrap();
        assert_eq!(back, entries);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn angles_wrap_and_clamp() {
        assert_eq!(angle_to_byte(0.0), 0);
        assert_eq!(angle_to_byte(360.0), 0);
        assert_eq!(angle_to_byte(-90.0), angle_to_byte(270.0));
        assert_eq!(pitch_to_byte(200.0), pitch_to_byte(90.0));
        assert_eq!(pitch_to_byte(-200.0), pitch_to_byte(-90.0));
    }
}
