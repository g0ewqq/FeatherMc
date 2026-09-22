use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};

use super::buffer::Buffer;
use super::error::NetworkError;
use super::packet::{frame_packet, RawPacket};

pub type ConnectionId = u64;

pub trait Transport: Send {
    fn try_read(&mut self, buf: &mut [u8]) -> io::Result<Option<usize>>;
    fn try_write(&mut self, buf: &[u8]) -> io::Result<Option<usize>>;
    fn flush(&mut self) -> io::Result<()>;
}

pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self { stream })
    }
}

fn no_progress(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    )
}

impl Transport for TcpTransport {
    fn try_read(&mut self, buf: &mut [u8]) -> io::Result<Option<usize>> {
        match self.stream.read(buf) {
            Ok(n) => Ok(Some(n)),
            Err(e) if no_progress(&e) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn try_write(&mut self, buf: &[u8]) -> io::Result<Option<usize>> {
        match self.stream.write(buf) {
            Ok(n) => Ok(Some(n)),
            Err(e) if no_progress(&e) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.stream.flush() {
            Ok(()) => Ok(()),
            Err(e) if no_progress(&e) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    Connected,
    Closing,
    Closed,
}

pub struct Connection {
    id: ConnectionId,
    peer: SocketAddr,
    state: ConnState,
    transport: Box<dyn Transport>,
    inbox: Buffer,
    outbox: Vec<u8>,
    bytes_read: u64,
    bytes_written: u64,
}

impl Connection {
    pub fn new(id: ConnectionId, peer: SocketAddr, transport: impl Transport + 'static) -> Self {
        Self {
            id,
            peer,
            state: ConnState::Connected,
            transport: Box::new(transport),
            inbox: Buffer::new(),
            outbox: Vec::new(),
            bytes_read: 0,
            bytes_written: 0,
        }
    }

    #[must_use]
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    #[must_use]
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    #[must_use]
    pub fn state(&self) -> ConnState {
        self.state
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.state == ConnState::Closed
    }

    #[must_use]
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    #[must_use]
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        if self.state != ConnState::Connected {
            return Err(NetworkError::ConnectionClosed);
        }
        self.outbox.extend_from_slice(data);
        self.flush_outbox();
        Ok(())
    }

    pub fn close(&mut self) {
        if self.state == ConnState::Connected {
            self.state = if self.outbox.is_empty() {
                ConnState::Closed
            } else {
                ConnState::Closing
            };
        }
    }

    pub fn poll(&mut self) {
        if self.state == ConnState::Closed {
            return;
        }
        self.read_available();
        if self.state == ConnState::Closed {
            return;
        }
        self.flush_outbox();
        if self.state == ConnState::Closing && self.outbox.is_empty() {
            self.state = ConnState::Closed;
        }
    }

    #[must_use]
    pub fn next_packet(&mut self) -> Option<RawPacket> {
        frame_packet(&mut self.inbox)
    }

    pub fn drain_available(&mut self) -> Vec<u8> {
        let len = self.inbox.len();
        self.inbox.take(len).unwrap_or_default()
    }

    fn read_available(&mut self) {
        let mut chunk = [0u8; 4096];
        match self.transport.try_read(&mut chunk) {
            Ok(Some(0)) => self.state = ConnState::Closed,
            Ok(Some(n)) => {
                self.inbox.extend(&chunk[..n]);
                self.bytes_read += n as u64;
            }
            Ok(None) => {}
            Err(_) => self.state = ConnState::Closed,
        }
    }

    fn flush_outbox(&mut self) {
        while !self.outbox.is_empty() {
            match self.transport.try_write(&self.outbox) {
                Ok(Some(0)) => break,
                Ok(Some(n)) => {
                    self.outbox.drain(..n);
                    self.bytes_written += n as u64;
                }
                Ok(None) => break,
                Err(_) => {
                    self.state = ConnState::Closed;
                    return;
                }
            }
        }
        if self.transport.flush().is_err() {
            self.state = ConnState::Closed;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use super::*;

    struct MemoryTransport {
        incoming: VecDeque<u8>,
        outgoing: Vec<u8>,
        open: bool,
    }

    #[derive(Clone)]
    struct MemoryHandle {
        inner: Arc<Mutex<MemoryTransport>>,
    }

    impl MemoryHandle {
        fn new() -> Self {
            Self {
                inner: Arc::new(Mutex::new(MemoryTransport {
                    incoming: VecDeque::new(),
                    outgoing: Vec::new(),
                    open: true,
                })),
            }
        }

        fn push_incoming(&self, bytes: &[u8]) {
            self.inner.lock().unwrap().incoming.extend(bytes);
        }

        fn take_outgoing(&self) -> Vec<u8> {
            std::mem::take(&mut self.inner.lock().unwrap().outgoing)
        }

        fn disconnect(&self) {
            self.inner.lock().unwrap().open = false;
        }
    }

    impl Transport for MemoryHandle {
        fn try_read(&mut self, buf: &mut [u8]) -> io::Result<Option<usize>> {
            let mut inner = self.inner.lock().unwrap();
            if !inner.incoming.is_empty() {
                let n = buf.len().min(inner.incoming.len());
                for slot in buf.iter_mut().take(n) {
                    *slot = inner.incoming.pop_front().unwrap();
                }
                return Ok(Some(n));
            }
            if inner.open {
                return Ok(None);
            }
            Ok(Some(0))
        }

        fn try_write(&mut self, buf: &[u8]) -> io::Result<Option<usize>> {
            let mut inner = self.inner.lock().unwrap();
            if !inner.open {
                return Err(io::Error::from(io::ErrorKind::ConnectionReset));
            }
            inner.outgoing.extend_from_slice(buf);
            Ok(Some(buf.len()))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn test_connection() -> (MemoryHandle, Connection) {
        let handle = MemoryHandle::new();
        let peer: SocketAddr = "127.0.0.1:1234".parse().unwrap();
        let conn = Connection::new(1, peer, handle.clone());
        (handle, conn)
    }

    fn framed(payload: &[u8]) -> Vec<u8> {
        let mut out = (payload.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn starts_connected_with_identity() {
        let (_handle, conn) = test_connection();
        assert_eq!(conn.id(), 1);
        assert_eq!(conn.peer(), "127.0.0.1:1234".parse().unwrap());
        assert_eq!(conn.state(), ConnState::Connected);
        assert!(!conn.is_closed());
    }

    #[test]
    fn send_flushes_and_counts_bytes() {
        let (handle, mut conn) = test_connection();
        conn.send(b"hello").unwrap();
        assert_eq!(handle.take_outgoing(), b"hello");
        assert_eq!(conn.bytes_written(), 5);
    }

    #[test]
    fn close_with_empty_outbox_goes_straight_to_closed() {
        let (_handle, mut conn) = test_connection();
        conn.close();
        assert_eq!(conn.state(), ConnState::Closed);
    }

    #[test]
    fn send_after_close_fails() {
        let (_handle, mut conn) = test_connection();
        conn.close();
        assert!(matches!(
            conn.send(b"hello"),
            Err(NetworkError::ConnectionClosed)
        ));
    }

    #[test]
    fn closed_state_never_reopens() {
        let (_handle, mut conn) = test_connection();
        conn.close();
        conn.poll();
        conn.close();
        assert_eq!(conn.state(), ConnState::Closed);
        assert!(matches!(
            conn.send(b"hello"),
            Err(NetworkError::ConnectionClosed)
        ));
    }

    #[test]
    fn remote_disconnect_marks_closed() {
        let (handle, mut conn) = test_connection();
        handle.disconnect();
        conn.poll();
        assert_eq!(conn.state(), ConnState::Closed);
    }

    #[test]
    fn receives_and_frames_packets() {
        let (handle, mut conn) = test_connection();
        handle.push_incoming(&framed(b"ping"));
        conn.poll();
        assert_eq!(
            conn.next_packet().map(RawPacket::into_data),
            Some(b"ping".to_vec())
        );
        assert_eq!(conn.next_packet(), None);
        assert_eq!(conn.bytes_read(), 8);
    }

    #[test]
    fn incomplete_frame_waits_for_more_bytes() {
        let (handle, mut conn) = test_connection();
        let bytes = framed(b"split");
        handle.push_incoming(&bytes[..3]);
        conn.poll();
        assert_eq!(conn.next_packet(), None);
        handle.push_incoming(&bytes[3..]);
        conn.poll();
        assert_eq!(
            conn.next_packet().map(RawPacket::into_data),
            Some(b"split".to_vec())
        );
    }
}
