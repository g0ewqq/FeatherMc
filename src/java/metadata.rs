use super::error::ProtoError;
use super::packets::write_slot;
use super::proto::{Reader, Writer};
use crate::inventory::ItemStack;

pub const FLAG_ON_FIRE: u8 = 0x01;
pub const FLAG_SNEAKING: u8 = 0x02;
pub const FLAG_SPRINTING: u8 = 0x08;
pub const FLAG_INVISIBLE: u8 = 0x20;
pub const FLAG_GLOWING: u8 = 0x40;

pub const POSE_STANDING: i32 = 0;
pub const POSE_CROUCHING: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetadataKind {
    Byte(u8),
    VarInt(i32),
    Bool(bool),
    Slot(ItemStack),
    Pose(i32),
}

impl MetadataKind {
    #[must_use]
    pub fn type_id(self) -> i32 {
        match self {
            MetadataKind::Byte(_) => 0,
            MetadataKind::VarInt(_) => 1,
            MetadataKind::Slot(_) => 7,
            MetadataKind::Bool(_) => 8,
            MetadataKind::Pose(_) => 20,
        }
    }

    fn encode(self, writer: &mut Writer) {
        match self {
            MetadataKind::Byte(value) => writer.write_u8(value),
            MetadataKind::VarInt(value) => writer.write_varint(value),
            MetadataKind::Slot(stack) => write_slot(writer, stack),
            MetadataKind::Bool(value) => writer.write_bool(value),
            MetadataKind::Pose(value) => writer.write_varint(value),
        }
    }

    fn decode(type_id: i32, reader: &mut Reader) -> Result<Self, ProtoError> {
        match type_id {
            0 => Ok(MetadataKind::Byte(reader.read_u8()?)),
            1 => Ok(MetadataKind::VarInt(reader.read_varint()?)),
            7 => Ok(MetadataKind::Slot(decode_slot(reader)?)),
            8 => Ok(MetadataKind::Bool(reader.read_bool()?)),
            20 => Ok(MetadataKind::Pose(reader.read_varint()?)),
            _ => Err(ProtoError::InvalidTag(type_id as u8)),
        }
    }
}

fn decode_slot(reader: &mut Reader) -> Result<ItemStack, ProtoError> {
    use crate::inventory::Item;

    let count = reader.read_varint()?;
    if count <= 0 {
        return Ok(ItemStack::empty());
    }
    let id = reader.read_varint()?;
    let added = reader.read_varint()?;
    if added != 0 {
        return Err(ProtoError::InvalidTag(7));
    }
    let removed = reader.read_varint()?;
    if removed != 0 {
        return Err(ProtoError::InvalidTag(7));
    }
    let item = Item::from_id(id).ok_or(ProtoError::InvalidTag(7))?;
    Ok(ItemStack {
        item,
        count: count as u32,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetadataEntry {
    pub index: u8,
    pub kind: MetadataKind,
}

pub fn encode_metadata(entries: &[MetadataEntry], writer: &mut Writer) {
    for entry in entries {
        writer.write_u8(entry.index);
        writer.write_varint(entry.kind.type_id());
        entry.kind.encode(writer);
    }
    writer.write_u8(0xFF);
}

pub fn decode_metadata(reader: &mut Reader) -> Result<Vec<MetadataEntry>, ProtoError> {
    let mut entries = Vec::new();
    loop {
        let index = reader.read_u8()?;
        if index == 0xFF {
            return Ok(entries);
        }
        let type_id = reader.read_varint()?;
        entries.push(MetadataEntry {
            index,
            kind: MetadataKind::decode(type_id, reader)?,
        });
    }
}

#[must_use]
pub fn player_metadata(flags: u8, pose: i32) -> Vec<MetadataEntry> {
    vec![
        MetadataEntry {
            index: 0,
            kind: MetadataKind::Byte(flags),
        },
        MetadataEntry {
            index: 6,
            kind: MetadataKind::Pose(pose),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(entries: &[MetadataEntry]) -> Vec<MetadataEntry> {
        let mut writer = Writer::new();
        encode_metadata(entries, &mut writer);
        let bytes = writer.into_bytes();
        decode_metadata(&mut Reader::new(&bytes)).unwrap()
    }

    #[test]
    fn encodes_player_metadata_with_terminator() {
        let entries = player_metadata(FLAG_SNEAKING, POSE_CROUCHING);
        let mut writer = Writer::new();
        encode_metadata(&entries, &mut writer);
        let bytes = writer.into_bytes();
        assert_eq!(bytes, vec![0x00, 0x00, 0x02, 0x06, 0x14, 0x05, 0xFF]);
        assert_eq!(roundtrip(&entries), entries);
    }

    #[test]
    fn roundtrips_every_supported_kind() {
        let entries = vec![
            MetadataEntry {
                index: 0,
                kind: MetadataKind::Byte(0x42),
            },
            MetadataEntry {
                index: 1,
                kind: MetadataKind::VarInt(-300),
            },
            MetadataEntry {
                index: 4,
                kind: MetadataKind::Bool(true),
            },
            MetadataEntry {
                index: 6,
                kind: MetadataKind::Pose(POSE_STANDING),
            },
        ];
        assert_eq!(roundtrip(&entries), entries);
    }

    #[test]
    fn slot_kind_roundtrips_componentless_stacks() {
        use crate::inventory::{Item, ItemStack};

        let entries = vec![MetadataEntry {
            index: 8,
            kind: MetadataKind::Slot(ItemStack::new(Item::Dirt, 3).unwrap()),
        }];
        assert_eq!(roundtrip(&entries), entries);

        let mut writer = Writer::new();
        writer.write_u8(8);
        writer.write_varint(7);
        writer.write_varint(2);
        writer.write_varint(1);
        writer.write_varint(0);
        writer.write_varint(0);
        writer.write_u8(0xFF);
        let bytes = writer.into_bytes();
        assert_eq!(
            decode_metadata(&mut Reader::new(&bytes)).unwrap(),
            vec![MetadataEntry {
                index: 8,
                kind: MetadataKind::Slot(ItemStack::new(Item::Stone, 2).unwrap()),
            }]
        );

        let mut writer = Writer::new();
        writer.write_u8(8);
        writer.write_varint(7);
        writer.write_varint(1);
        writer.write_varint(1);
        writer.write_varint(1);
        writer.write_varint(0);
        writer.write_u8(0xFF);
        let bytes = writer.into_bytes();
        assert!(decode_metadata(&mut Reader::new(&bytes)).is_err());
    }

    #[test]
    fn empty_metadata_is_just_a_terminator() {
        let mut writer = Writer::new();
        encode_metadata(&[], &mut writer);
        assert_eq!(writer.into_bytes(), vec![0xFF]);
    }

    #[test]
    fn rejects_unknown_serializer() {
        let mut writer = Writer::new();
        writer.write_u8(3);
        writer.write_varint(99);
        writer.write_u8(0xFF);
        let bytes = writer.into_bytes();
        assert_eq!(
            decode_metadata(&mut Reader::new(&bytes)),
            Err(ProtoError::InvalidTag(99))
        );
    }
}
