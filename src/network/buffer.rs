#[derive(Debug, Default)]
pub struct Buffer {
    data: Vec<u8>,
    start: usize,
}

impl Buffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len() - self.start
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.data.len()
    }

    pub fn extend(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    #[must_use]
    pub fn peek(&self, n: usize) -> Option<&[u8]> {
        self.data.get(self.start..self.start.checked_add(n)?)
    }

    pub fn consume(&mut self, n: usize) {
        self.start += n;
        debug_assert!(self.start <= self.data.len());
        if self.start >= 4096 && self.start * 2 >= self.data.len() {
            self.data.drain(..self.start);
            self.start = 0;
        }
        if self.is_empty() {
            self.data.clear();
            self.start = 0;
        }
    }

    pub fn take(&mut self, n: usize) -> Option<Vec<u8>> {
        let bytes = self.peek(n)?.to_vec();
        self.consume(n);
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_does_not_consume() {
        let mut buf = Buffer::new();
        assert!(buf.is_empty());
        buf.extend(b"hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.peek(5), Some(b"hello".as_slice()));
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn take_returns_bytes_in_order() {
        let mut buf = Buffer::new();
        buf.extend(b"ab");
        buf.extend(b"cd");
        assert_eq!(buf.take(3), Some(b"abc".to_vec()));
        assert_eq!(buf.take(1), Some(b"d".to_vec()));
        assert!(buf.is_empty());
    }

    #[test]
    fn short_reads_return_none_and_keep_data() {
        let mut buf = Buffer::new();
        buf.extend(b"ab");
        assert_eq!(buf.peek(3), None);
        assert_eq!(buf.take(3), None);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn stays_consistent_across_compaction() {
        let mut buf = Buffer::new();
        buf.extend(&vec![7u8; 5000]);
        assert_eq!(buf.take(4096), Some(vec![7u8; 4096]));
        assert_eq!(buf.len(), 904);
        assert_eq!(buf.take(904), Some(vec![7u8; 904]));
        assert!(buf.is_empty());
        buf.extend(b"after");
        assert_eq!(buf.take(5), Some(b"after".to_vec()));
    }
}
