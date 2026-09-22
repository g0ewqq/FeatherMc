use super::buffer::Buffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPacket(pub Vec<u8>);

impl RawPacket {
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn into_data(self) -> Vec<u8> {
        self.0
    }
}

#[must_use]
pub fn frame_packet(buf: &mut Buffer) -> Option<RawPacket> {
    let len = u32::from_be_bytes(buf.peek(4)?.try_into().ok()?) as usize;
    let mut bytes = buf.take(len.checked_add(4)?)?;
    bytes.drain(..4);
    Some(RawPacket::new(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn framed(payload: &[u8]) -> Vec<u8> {
        let mut out = (payload.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn frames_single_packet() {
        let mut buf = Buffer::new();
        buf.extend(&framed(b"hello"));
        assert_eq!(
            frame_packet(&mut buf),
            Some(RawPacket::new(b"hello".to_vec()))
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn waits_for_split_header_and_body() {
        let mut buf = Buffer::new();
        let bytes = framed(b"split-packet");
        buf.extend(&bytes[..2]);
        assert_eq!(frame_packet(&mut buf), None);
        buf.extend(&bytes[2..7]);
        assert_eq!(frame_packet(&mut buf), None);
        buf.extend(&bytes[7..]);
        assert_eq!(
            frame_packet(&mut buf),
            Some(RawPacket::new(b"split-packet".to_vec()))
        );
    }

    #[test]
    fn frames_back_to_back_packets() {
        let mut buf = Buffer::new();
        buf.extend(&framed(b"one"));
        buf.extend(&framed(b""));
        buf.extend(&framed(b"three"));
        assert_eq!(
            frame_packet(&mut buf),
            Some(RawPacket::new(b"one".to_vec()))
        );
        assert_eq!(frame_packet(&mut buf), Some(RawPacket::new(vec![])));
        assert_eq!(
            frame_packet(&mut buf),
            Some(RawPacket::new(b"three".to_vec()))
        );
        assert_eq!(frame_packet(&mut buf), None);
    }

    #[test]
    fn ignores_trailing_garbage_until_complete() {
        let mut buf = Buffer::new();
        buf.extend(b"\x00\x00");
        assert_eq!(frame_packet(&mut buf), None);
        assert_eq!(buf.len(), 2);
    }
}
