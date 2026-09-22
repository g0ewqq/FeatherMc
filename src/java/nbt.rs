use std::collections::BTreeMap;

use super::error::ProtoError;
use super::proto::{Reader, Writer};

const MAX_DEPTH: u32 = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtTag>),
    Compound(BTreeMap<String, NbtTag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtTag {
    #[must_use]
    pub fn id(&self) -> u8 {
        match self {
            NbtTag::Byte(_) => 1,
            NbtTag::Short(_) => 2,
            NbtTag::Int(_) => 3,
            NbtTag::Long(_) => 4,
            NbtTag::Float(_) => 5,
            NbtTag::Double(_) => 6,
            NbtTag::ByteArray(_) => 7,
            NbtTag::String(_) => 8,
            NbtTag::List(_) => 9,
            NbtTag::Compound(_) => 10,
            NbtTag::IntArray(_) => 11,
            NbtTag::LongArray(_) => 12,
        }
    }

    #[must_use]
    pub fn compound(entries: Vec<(String, NbtTag)>) -> Self {
        NbtTag::Compound(entries.into_iter().collect())
    }

    pub fn encode_named(&self, writer: &mut Writer, name: &str) -> Result<(), ProtoError> {
        writer.write_u8(self.id());
        writer.write_u16_string(name);
        self.encode_payload(writer, 0)
    }

    pub fn encode_unnamed(&self, writer: &mut Writer) -> Result<(), ProtoError> {
        writer.write_u8(self.id());
        self.encode_payload(writer, 0)
    }

    fn encode_child(&self, writer: &mut Writer, name: &str, depth: u32) -> Result<(), ProtoError> {
        writer.write_u8(self.id());
        writer.write_u16_string(name);
        self.encode_payload(writer, depth)
    }

    fn encode_payload(&self, writer: &mut Writer, depth: u32) -> Result<(), ProtoError> {
        if depth > MAX_DEPTH {
            return Err(ProtoError::DepthExceeded);
        }
        match self {
            NbtTag::Byte(v) => writer.write_i8(*v),
            NbtTag::Short(v) => writer.write_i16(*v),
            NbtTag::Int(v) => writer.write_i32(*v),
            NbtTag::Long(v) => writer.write_i64(*v),
            NbtTag::Float(v) => writer.write_f32(*v),
            NbtTag::Double(v) => writer.write_f64(*v),
            NbtTag::ByteArray(bytes) => {
                writer.write_i32(bytes.len() as i32);
                for b in bytes {
                    writer.write_i8(*b);
                }
            }
            NbtTag::String(s) => writer.write_u16_string(s),
            NbtTag::List(items) => {
                let element_id = items.first().map(NbtTag::id).unwrap_or(0);
                if items.iter().any(|item| item.id() != element_id) {
                    return Err(ProtoError::MixedList);
                }
                writer.write_u8(element_id);
                writer.write_i32(items.len() as i32);
                for item in items {
                    item.encode_payload(writer, depth + 1)?;
                }
            }
            NbtTag::Compound(entries) => {
                for (name, tag) in entries {
                    tag.encode_child(writer, name, depth + 1)?;
                }
                writer.write_u8(0);
            }
            NbtTag::IntArray(values) => {
                writer.write_i32(values.len() as i32);
                for v in values {
                    writer.write_i32(*v);
                }
            }
            NbtTag::LongArray(values) => {
                writer.write_i32(values.len() as i32);
                for v in values {
                    writer.write_i64(*v);
                }
            }
        }
        Ok(())
    }

    pub fn decode_named(reader: &mut Reader) -> Result<(String, NbtTag), ProtoError> {
        let id = reader.read_u8()?;
        if id == 0 {
            return Err(ProtoError::InvalidTag(id));
        }
        let name = reader.read_u16_string()?;
        Ok((name, decode_payload(reader, id, 0)?))
    }

    pub fn decode_unnamed(reader: &mut Reader) -> Result<NbtTag, ProtoError> {
        let id = reader.read_u8()?;
        decode_payload(reader, id, 0)
    }
}

fn decode_payload(reader: &mut Reader, id: u8, depth: u32) -> Result<NbtTag, ProtoError> {
    if depth > MAX_DEPTH {
        return Err(ProtoError::DepthExceeded);
    }
    match id {
        1 => Ok(NbtTag::Byte(reader.read_i8()?)),
        2 => Ok(NbtTag::Short(reader.read_i16()?)),
        3 => Ok(NbtTag::Int(reader.read_i32()?)),
        4 => Ok(NbtTag::Long(reader.read_i64()?)),
        5 => Ok(NbtTag::Float(reader.read_f32()?)),
        6 => Ok(NbtTag::Double(reader.read_f64()?)),
        7 => {
            let len = read_len(reader)?;
            let bytes = reader.read_bytes(len)?;
            Ok(NbtTag::ByteArray(bytes.iter().map(|&b| b as i8).collect()))
        }
        8 => Ok(NbtTag::String(reader.read_u16_string()?)),
        9 => {
            let element_id = reader.read_u8()?;
            let len = read_len(reader)?;
            if element_id == 0 && len > 0 {
                return Err(ProtoError::InvalidTag(element_id));
            }
            let mut items = Vec::new();
            for _ in 0..len {
                items.push(decode_payload(reader, element_id, depth + 1)?);
            }
            Ok(NbtTag::List(items))
        }
        10 => {
            let mut entries = BTreeMap::new();
            loop {
                let id = reader.read_u8()?;
                if id == 0 {
                    break;
                }
                if id > 12 {
                    return Err(ProtoError::InvalidTag(id));
                }
                let name = reader.read_u16_string()?;
                entries.insert(name, decode_payload(reader, id, depth + 1)?);
            }
            Ok(NbtTag::Compound(entries))
        }
        11 => {
            let len = read_len(reader)?;
            let mut values = Vec::new();
            for _ in 0..len {
                values.push(reader.read_i32()?);
            }
            Ok(NbtTag::IntArray(values))
        }
        12 => {
            let len = read_len(reader)?;
            let mut values = Vec::new();
            for _ in 0..len {
                values.push(reader.read_i64()?);
            }
            Ok(NbtTag::LongArray(values))
        }
        id => Err(ProtoError::InvalidTag(id)),
    }
}

fn read_len(reader: &mut Reader) -> Result<usize, ProtoError> {
    let len = reader.read_i32()?;
    if len < 0 {
        return Err(ProtoError::NegativeLength(len));
    }
    Ok(len as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(tag: &NbtTag, name: &str) -> Vec<u8> {
        let mut writer = Writer::new();
        tag.encode_named(&mut writer, name).unwrap();
        writer.into_bytes()
    }

    fn decoded(bytes: &[u8]) -> (String, NbtTag) {
        NbtTag::decode_named(&mut Reader::new(bytes)).unwrap()
    }

    #[test]
    fn encodes_primitives_to_known_bytes() {
        assert_eq!(
            encoded(&NbtTag::Byte(5), "a"),
            vec![0x01, 0x00, 0x01, b'a', 0x05]
        );
        assert_eq!(
            encoded(&NbtTag::Short(-300), "b"),
            vec![0x02, 0x00, 0x01, b'b', 0xFE, 0xD4]
        );
        assert_eq!(
            encoded(&NbtTag::Int(0x01020304), ""),
            vec![0x03, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04]
        );
        assert_eq!(
            encoded(&NbtTag::Float(1.5), ""),
            vec![0x05, 0x00, 0x00, 0x3F, 0xC0, 0x00, 0x00]
        );
        assert_eq!(
            encoded(&NbtTag::String("hi".to_owned()), "greeting"),
            vec![
                0x08, 0x00, 0x08, b'g', b'r', b'e', b'e', b't', b'i', b'n', b'g', 0x00, 0x02, b'h',
                b'i'
            ]
        );
    }

    #[test]
    fn encodes_containers_to_known_bytes() {
        let list = NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)]);
        assert_eq!(
            encoded(&list, "l"),
            vec![
                0x09, 0x00, 0x01, b'l', 0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x02
            ]
        );

        let empty_list = NbtTag::List(vec![]);
        assert_eq!(
            encoded(&empty_list, "e"),
            vec![0x09, 0x00, 0x01, b'e', 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        let empty_compound = NbtTag::Compound(BTreeMap::new());
        assert_eq!(encoded(&empty_compound, ""), vec![0x0A, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn roundtrips_complex_structure() {
        let tag = NbtTag::compound(vec![
            ("byte".to_owned(), NbtTag::Byte(-128)),
            ("short".to_owned(), NbtTag::Short(32767)),
            ("int".to_owned(), NbtTag::Int(i32::MIN)),
            ("long".to_owned(), NbtTag::Long(i64::MAX)),
            ("float".to_owned(), NbtTag::Float(-0.5)),
            ("double".to_owned(), NbtTag::Double(1.0 / 3.0)),
            ("bytes".to_owned(), NbtTag::ByteArray(vec![-1, 0, 127])),
            ("text".to_owned(), NbtTag::String("Grüße 🎮".to_owned())),
            ("empty_text".to_owned(), NbtTag::String(String::new())),
            (
                "nested".to_owned(),
                NbtTag::compound(vec![
                    ("inner".to_owned(), NbtTag::Int(42)),
                    ("empty".to_owned(), NbtTag::Compound(BTreeMap::new())),
                ]),
            ),
            (
                "ints".to_owned(),
                NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(-2), NbtTag::Int(3)]),
            ),
            ("empty_list".to_owned(), NbtTag::List(vec![])),
            (
                "compounds".to_owned(),
                NbtTag::List(vec![
                    NbtTag::compound(vec![("x".to_owned(), NbtTag::Byte(1))]),
                    NbtTag::compound(vec![("y".to_owned(), NbtTag::Byte(2))]),
                ]),
            ),
            (
                "int_array".to_owned(),
                NbtTag::IntArray(vec![i32::MIN, 0, i32::MAX]),
            ),
            (
                "long_array".to_owned(),
                NbtTag::LongArray(vec![i64::MIN, i64::MAX]),
            ),
            ("empty_array".to_owned(), NbtTag::IntArray(vec![])),
        ]);
        let bytes = encoded(&tag, "root");
        let (name, back) = decoded(&bytes);
        assert_eq!(name, "root");
        assert_eq!(back, tag);
    }

    #[test]
    fn unnamed_form_has_no_name_bytes() {
        let tag = NbtTag::compound(vec![("a".to_owned(), NbtTag::Byte(5))]);
        let mut writer = Writer::new();
        tag.encode_unnamed(&mut writer).unwrap();
        assert_eq!(
            writer.into_bytes(),
            vec![0x0A, 0x01, 0x00, 0x01, b'a', 0x05, 0x00]
        );

        let mut writer = Writer::new();
        tag.encode_unnamed(&mut writer).unwrap();
        let bytes = writer.into_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(NbtTag::decode_unnamed(&mut reader).unwrap(), tag);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn rejects_mixed_lists() {
        let mixed = NbtTag::List(vec![NbtTag::Int(1), NbtTag::String("x".to_owned())]);
        let mut writer = Writer::new();
        assert_eq!(
            mixed.encode_named(&mut writer, "m"),
            Err(ProtoError::MixedList)
        );
    }

    #[test]
    fn rejects_malformed_input() {
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&[])),
            Err(ProtoError::Truncated)
        );
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&[13, 0x00, 0x00])),
            Err(ProtoError::InvalidTag(13))
        );
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&[0x03, 0x00, 0x00])),
            Err(ProtoError::Truncated)
        );

        let mut writer = Writer::new();
        writer.write_u8(11);
        writer.write_u16_string("");
        writer.write_i32(-4);
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&writer.into_bytes())),
            Err(ProtoError::NegativeLength(-4))
        );

        let mut writer = Writer::new();
        writer.write_u8(8);
        writer.write_u16_string("");
        writer.write_u16(2);
        writer.write_bytes(&[0xFF, 0xFF]);
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&writer.into_bytes())),
            Err(ProtoError::InvalidUtf8)
        );

        let mut writer = Writer::new();
        writer.write_u8(9);
        writer.write_u16_string("");
        writer.write_u8(3);
        writer.write_i32(100);
        writer.write_i32(1);
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&writer.into_bytes())),
            Err(ProtoError::Truncated)
        );
    }

    #[test]
    fn rejects_extreme_nesting() {
        let mut tag = NbtTag::Int(1);
        for i in 0..600 {
            tag = NbtTag::compound(vec![(format!("n{i}"), tag)]);
        }
        let mut writer = Writer::new();
        assert_eq!(
            tag.encode_named(&mut writer, "deep"),
            Err(ProtoError::DepthExceeded)
        );

        let mut bytes = vec![0x0A, 0x00, 0x00];
        for _ in 0..600 {
            bytes.push(0x0A);
            bytes.extend_from_slice(&[0x00, 0x01, b'k']);
        }
        bytes.push(0x03);
        bytes.extend_from_slice(&[0x00, 0x01, b'v', 0x00, 0x00, 0x00, 0x01]);
        bytes.resize(bytes.len() + 600, 0x00);
        assert_eq!(
            NbtTag::decode_named(&mut Reader::new(&bytes)),
            Err(ProtoError::DepthExceeded)
        );
    }
}
