use super::error::ProtoError;

pub const MAX_STRING_BYTES: i32 = 32767;

pub fn decode_varint_prefix(bytes: &[u8]) -> Result<Option<(i32, usize)>, ProtoError> {
    let mut value = 0u32;
    for i in 0..5 {
        let byte = match bytes.get(i) {
            Some(&byte) => byte,
            None => return Ok(None),
        };
        value = value.wrapping_add(((byte & 0x7F) as u32).wrapping_shl(7 * i as u32));
        if byte & 0x80 == 0 {
            return Ok(Some((value as i32, i + 1)));
        }
    }
    Err(ProtoError::VarIntTooLong)
}

fn decode_varlong_prefix(bytes: &[u8]) -> Result<Option<(i64, usize)>, ProtoError> {
    let mut value = 0u64;
    for i in 0..10 {
        let byte = match bytes.get(i) {
            Some(&byte) => byte,
            None => return Ok(None),
        };
        value = value.wrapping_add(((byte & 0x7F) as u64).wrapping_shl(7 * i as u32));
        if byte & 0x80 == 0 {
            return Ok(Some((value as i64, i + 1)));
        }
    }
    Err(ProtoError::VarLongTooLong)
}

fn sign_extend(value: u64, bits: u32) -> i32 {
    (((value << (64 - bits)) as i64) >> (64 - bits)) as i32
}

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    #[must_use]
    pub fn position(&self) -> usize {
        self.pos
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ProtoError> {
        if self.remaining() < n {
            return Err(ProtoError::Truncated);
        }
        let bytes = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(bytes)
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], ProtoError> {
        self.take(n)
    }

    pub fn read_u16_string(&mut self) -> Result<String, ProtoError> {
        let len = self.read_u16()? as usize;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ProtoError::InvalidUtf8)
    }

    pub fn read_u8(&mut self) -> Result<u8, ProtoError> {
        Ok(self.take(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, ProtoError> {
        Ok(self.take(1)?[0] as i8)
    }

    pub fn read_bool(&mut self) -> Result<bool, ProtoError> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16(&mut self) -> Result<u16, ProtoError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn read_i16(&mut self) -> Result<i16, ProtoError> {
        Ok(i16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn read_u32(&mut self) -> Result<u32, ProtoError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn read_i32(&mut self) -> Result<i32, ProtoError> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn read_u64(&mut self) -> Result<u64, ProtoError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn read_i64(&mut self) -> Result<i64, ProtoError> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn read_f32(&mut self) -> Result<f32, ProtoError> {
        Ok(f32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn read_f64(&mut self) -> Result<f64, ProtoError> {
        Ok(f64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn read_varint(&mut self) -> Result<i32, ProtoError> {
        match decode_varint_prefix(&self.data[self.pos..])? {
            Some((value, len)) => {
                self.pos += len;
                Ok(value)
            }
            None => Err(ProtoError::Truncated),
        }
    }

    pub fn read_varlong(&mut self) -> Result<i64, ProtoError> {
        match decode_varlong_prefix(&self.data[self.pos..])? {
            Some((value, len)) => {
                self.pos += len;
                Ok(value)
            }
            None => Err(ProtoError::Truncated),
        }
    }

    pub fn read_string(&mut self) -> Result<String, ProtoError> {
        let len = self.read_varint()?;
        if !(0..=MAX_STRING_BYTES).contains(&len) {
            return Err(ProtoError::StringTooLong(len));
        }
        let bytes = self.take(len as usize)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ProtoError::InvalidUtf8)
    }

    pub fn read_uuid(&mut self) -> Result<u128, ProtoError> {
        Ok(u128::from_be_bytes(self.take(16)?.try_into().unwrap()))
    }

    pub fn read_position(&mut self) -> Result<(i32, i32, i32), ProtoError> {
        let packed = self.read_u64()?;
        Ok((
            sign_extend(packed >> 38, 26),
            sign_extend(packed, 12),
            sign_extend(packed >> 12, 26),
        ))
    }
}

#[derive(Debug, Default)]
pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.data.push(u8::from(value));
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i16(&mut self, value: i16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_f32(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_f64(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_varint(&mut self, value: i32) {
        let mut value = value as u32;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                self.data.push(byte);
                return;
            }
            self.data.push(byte | 0x80);
        }
    }

    pub fn write_varlong(&mut self, value: i64) {
        let mut value = value as u64;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value == 0 {
                self.data.push(byte);
                return;
            }
            self.data.push(byte | 0x80);
        }
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_varint(value.len() as i32);
        self.data.extend_from_slice(value.as_bytes());
    }

    pub fn write_u16_string(&mut self, value: &str) {
        debug_assert!(value.len() <= u16::MAX as usize);
        self.write_u16(value.len() as u16);
        self.data.extend_from_slice(value.as_bytes());
    }

    pub fn write_uuid(&mut self, value: u128) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_position(&mut self, x: i32, y: i32, z: i32) {
        let packed =
            ((x as u64 & 0x3FF_FFFF) << 38) | ((z as u64 & 0x3FF_FFFF) << 12) | (y as u64 & 0xFFF);
        self.write_u64(packed);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint_roundtrip(value: i32) {
        let mut writer = Writer::new();
        writer.write_varint(value);
        let bytes = writer.into_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_varint().unwrap(), value);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn varint_encodings() {
        let mut writer = Writer::new();
        writer.write_varint(0);
        assert_eq!(writer.into_bytes(), vec![0x00]);

        let mut writer = Writer::new();
        writer.write_varint(1);
        assert_eq!(writer.into_bytes(), vec![0x01]);

        let mut writer = Writer::new();
        writer.write_varint(127);
        assert_eq!(writer.into_bytes(), vec![0x7F]);

        let mut writer = Writer::new();
        writer.write_varint(128);
        assert_eq!(writer.into_bytes(), vec![0x80, 0x01]);

        let mut writer = Writer::new();
        writer.write_varint(25565);
        assert_eq!(writer.into_bytes(), vec![0xDD, 0xC7, 0x01]);

        let mut writer = Writer::new();
        writer.write_varint(-1);
        assert_eq!(writer.into_bytes(), vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn varint_roundtrips() {
        for value in [
            0,
            1,
            2,
            127,
            128,
            255,
            25565,
            2097151,
            i32::MAX,
            -1,
            -2,
            i32::MIN,
        ] {
            varint_roundtrip(value);
        }
    }

    #[test]
    fn varint_rejects_overlong_input() {
        assert_eq!(
            decode_varint_prefix(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]),
            Err(ProtoError::VarIntTooLong)
        );
        assert_eq!(decode_varint_prefix(&[]), Ok(None));
        assert_eq!(decode_varint_prefix(&[0x80]), Ok(None));
    }

    #[test]
    fn varint_read_fails_on_truncated_input() {
        let mut reader = Reader::new(&[0x80]);
        assert_eq!(reader.read_varint(), Err(ProtoError::Truncated));
    }

    fn varlong_roundtrip(value: i64) {
        let mut writer = Writer::new();
        writer.write_varlong(value);
        let bytes = writer.into_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_varlong().unwrap(), value);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn varlong_roundtrips() {
        for value in [
            0,
            1,
            127,
            128,
            25565,
            i32::MAX as i64,
            i64::MAX,
            -1,
            i64::MIN,
        ] {
            varlong_roundtrip(value);
        }
    }

    #[test]
    fn varlong_rejects_overlong_input() {
        assert_eq!(
            decode_varlong_prefix(&[0x80; 11]),
            Err(ProtoError::VarLongTooLong)
        );
        assert_eq!(decode_varlong_prefix(&[]), Ok(None));
        let mut reader = Reader::new(&[0x80, 0x80]);
        assert_eq!(reader.read_varlong(), Err(ProtoError::Truncated));
    }

    #[test]
    fn primitives_roundtrip() {
        let mut writer = Writer::new();
        writer.write_u8(0xAB);
        writer.write_i8(-5);
        writer.write_bool(true);
        writer.write_bool(false);
        writer.write_u16(0xCDEF);
        writer.write_i16(-300);
        writer.write_u32(0xDEAD_BEEF);
        writer.write_i32(-123456);
        writer.write_u64(0x0123_4567_89AB_CDEF);
        writer.write_i64(-9876543210);
        writer.write_f32(1.5);
        writer.write_f64(-2.25);
        let bytes = writer.into_bytes();

        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_u8().unwrap(), 0xAB);
        assert_eq!(reader.read_i8().unwrap(), -5);
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_u16().unwrap(), 0xCDEF);
        assert_eq!(reader.read_i16().unwrap(), -300);
        assert_eq!(reader.read_u32().unwrap(), 0xDEAD_BEEF);
        assert_eq!(reader.read_i32().unwrap(), -123456);
        assert_eq!(reader.read_u64().unwrap(), 0x0123_4567_89AB_CDEF);
        assert_eq!(reader.read_i64().unwrap(), -9876543210);
        assert_eq!(reader.read_f32().unwrap(), 1.5);
        assert_eq!(reader.read_f64().unwrap(), -2.25);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn bool_treats_nonzero_as_true() {
        let mut reader = Reader::new(&[0, 1, 2]);
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
    }

    #[test]
    fn strings_roundtrip() {
        for text in ["", "hello", "A FeatherMC Server", "Grüße 🎮 こんにちは"] {
            let mut writer = Writer::new();
            writer.write_string(text);
            let bytes = writer.into_bytes();
            let mut reader = Reader::new(&bytes);
            assert_eq!(reader.read_string().unwrap(), text);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn strings_reject_bad_input() {
        let mut writer = Writer::new();
        writer.write_varint(MAX_STRING_BYTES + 1);
        let bytes = writer.into_bytes();
        assert_eq!(
            Reader::new(&bytes).read_string(),
            Err(ProtoError::StringTooLong(MAX_STRING_BYTES + 1))
        );

        let mut writer = Writer::new();
        writer.write_varint(-1);
        let bytes = writer.into_bytes();
        assert_eq!(
            Reader::new(&bytes).read_string(),
            Err(ProtoError::StringTooLong(-1))
        );

        let mut writer = Writer::new();
        writer.write_varint(3);
        writer.write_bytes(&[0xFF, 0xFF, 0xFF]);
        let bytes = writer.into_bytes();
        assert_eq!(
            Reader::new(&bytes).read_string(),
            Err(ProtoError::InvalidUtf8)
        );

        let mut writer = Writer::new();
        writer.write_varint(10);
        writer.write_bytes(b"short");
        let bytes = writer.into_bytes();
        assert_eq!(
            Reader::new(&bytes).read_string(),
            Err(ProtoError::Truncated)
        );
    }

    #[test]
    fn positions_roundtrip() {
        for (x, y, z) in [
            (0, 100, 0),
            (8, 64, 8),
            (-1, -64, -1),
            (33_554_431, 2047, -33_554_432),
            (i32::MIN >> 6, i32::MIN >> 20, i32::MIN >> 6),
        ] {
            let mut writer = Writer::new();
            writer.write_position(x, y, z);
            let bytes = writer.into_bytes();
            assert_eq!(bytes.len(), 8);
            assert_eq!(Reader::new(&bytes).read_position().unwrap(), (x, y, z));
        }
    }

    #[test]
    fn origin_packs_to_known_bytes() {
        let mut writer = Writer::new();
        writer.write_position(0, 100, 0);
        assert_eq!(writer.into_bytes(), vec![0, 0, 0, 0, 0, 0, 0, 100]);
    }

    #[test]
    fn uuid_roundtrips() {
        let mut writer = Writer::new();
        writer.write_uuid(0x123E_4567_E89B_12D3_A456_4266_1417_4000);
        let bytes = writer.into_bytes();
        let mut reader = Reader::new(&bytes);
        assert_eq!(
            reader.read_uuid().unwrap(),
            0x123E_4567_E89B_12D3_A456_4266_1417_4000
        );

        let mut reader = Reader::new(&[0u8; 15]);
        assert_eq!(reader.read_uuid(), Err(ProtoError::Truncated));
    }
}
