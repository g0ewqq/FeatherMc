use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};

use tracing::{debug, info};

use super::connection::{Connection, ConnectionId, TcpTransport};
use super::error::NetworkError;

const MAX_ACCEPTS_PER_TICK: u32 = 64;

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkMetrics {
    pub accepted: u64,
    pub disconnected: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

pub struct NetworkManager {
    listener: Option<TcpListener>,
    connections: HashMap<ConnectionId, Connection>,
    next_id: ConnectionId,
    metrics: NetworkMetrics,
    max_connections: Option<usize>,
}

impl NetworkManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            listener: None,
            connections: HashMap::new(),
            next_id: 1,
            metrics: NetworkMetrics::default(),
            max_connections: None,
        }
    }

    /// Cap on accepted connections so idle sockets can't grow memory forever.
    /// At the limit, new sockets sit in the OS backlog until a slot frees up.
    /// `None` (the default) means no cap at all.
    pub fn set_max_connections(&mut self, max: Option<usize>) {
        self.max_connections = max;
    }

    pub fn bind(&mut self, addr: SocketAddr) -> Result<(), NetworkError> {
        if self.listener.is_some() {
            return Err(NetworkError::AlreadyBound);
        }
        let listener =
            TcpListener::bind(addr).map_err(|source| NetworkError::Bind { addr, source })?;
        listener
            .set_nonblocking(true)
            .map_err(|source| NetworkError::Bind { addr, source })?;
        self.listener = Some(listener);
        Ok(())
    }

    #[must_use]
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.listener.as_ref()?.local_addr().ok()
    }

    pub fn poll(&mut self) {
        self.accept_pending();

        let mut closed = Vec::new();
        for conn in self.connections.values_mut() {
            conn.poll();
            if conn.is_closed() {
                closed.push((conn.id(), conn.bytes_read(), conn.bytes_written()));
            }
        }
        for (id, read, written) in closed {
            self.connections.remove(&id);
            self.metrics.disconnected += 1;
            self.metrics.bytes_read += read;
            self.metrics.bytes_written += written;
            debug!("connection {id} removed");
        }
    }

    pub fn shutdown(&mut self) {
        self.listener = None;
        if !self.connections.is_empty() {
            info!("Closing {} active connection(s)", self.connections.len());
        }
        for (_, conn) in self.connections.drain() {
            self.metrics.disconnected += 1;
            self.metrics.bytes_read += conn.bytes_read();
            self.metrics.bytes_written += conn.bytes_written();
        }
    }

    #[must_use]
    pub fn connection(&self, id: ConnectionId) -> Option<&Connection> {
        self.connections.get(&id)
    }

    pub fn connection_mut(&mut self, id: ConnectionId) -> Option<&mut Connection> {
        self.connections.get_mut(&id)
    }

    pub fn remove(&mut self, id: ConnectionId) -> bool {
        match self.connections.remove(&id) {
            Some(conn) => {
                self.metrics.disconnected += 1;
                self.metrics.bytes_read += conn.bytes_read();
                self.metrics.bytes_written += conn.bytes_written();
                true
            }
            None => false,
        }
    }

    pub fn connections(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    #[must_use]
    pub fn metrics(&self) -> NetworkMetrics {
        let mut total = self.metrics;
        for conn in self.connections.values() {
            total.bytes_read += conn.bytes_read();
            total.bytes_written += conn.bytes_written();
        }
        total
    }

    fn accept_pending(&mut self) {
        for _ in 0..MAX_ACCEPTS_PER_TICK {
            if self
                .max_connections
                .is_some_and(|max| self.connections.len() >= max)
            {
                break;
            }
            let next = match self.listener.as_ref() {
                None => return,
                Some(listener) => match listener.accept() {
                    Ok((stream, addr)) => Some(Ok((stream, addr))),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => None,
                    Err(e) => Some(Err(e)),
                },
            };
            match next {
                Some(Ok((stream, addr))) => self.register(stream, addr),
                Some(Err(e)) => {
                    debug!("accept failed: {e}");
                    break;
                }
                None => break,
            }
        }
    }

    fn register(&mut self, stream: TcpStream, addr: SocketAddr) {
        let id = self.next_id;
        self.next_id += 1;
        match TcpTransport::new(stream) {
            Ok(transport) => {
                self.connections
                    .insert(id, Connection::new(id, addr, transport));
                self.metrics.accepted += 1;
                debug!("accepted connection {id} from {addr}");
            }
            Err(e) => debug!("dropping connection from {addr}: {e}"),
        }
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;

    fn test_manager() -> (NetworkManager, SocketAddr) {
        let mut manager = NetworkManager::new();
        manager.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = manager.local_addr().unwrap();
        (manager, addr)
    }

    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut out = (payload.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(payload);
        out
    }

    fn wait_until(manager: &mut NetworkManager, mut done: impl FnMut(&NetworkManager) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !done(manager) {
            assert!(Instant::now() < deadline, "timed out waiting for server");
            manager.poll();
            thread::sleep(Duration::from_millis(2));
        }
        manager.poll();
    }

    fn recv_packet(manager: &mut NetworkManager, id: ConnectionId) -> Vec<u8> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            manager.poll();
            if let Some(conn) = manager.connection_mut(id) {
                if let Some(packet) = conn.next_packet() {
                    return packet.into_data();
                }
            }
            assert!(Instant::now() < deadline, "timed out waiting for packet");
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn connect(addr: SocketAddr) -> TcpStream {
        let client = TcpStream::connect(addr).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        client
    }

    #[test]
    fn rejects_second_bind() {
        let (mut manager, addr) = test_manager();
        assert!(matches!(
            manager.bind(addr),
            Err(NetworkError::AlreadyBound)
        ));
    }

    #[test]
    fn accepts_and_registers_connection() {
        let (mut manager, addr) = test_manager();
        let client = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        let conn = manager.connection(1).unwrap();
        assert_eq!(conn.peer(), client.local_addr().unwrap());
        assert_eq!(manager.metrics().accepted, 1);
    }

    #[test]
    fn receives_framed_bytes() {
        let (mut manager, addr) = test_manager();
        let mut client = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        client.write_all(&frame(b"hello-spiron")).unwrap();
        assert_eq!(recv_packet(&mut manager, 1), b"hello-spiron");
        assert!(manager.metrics().bytes_read >= 15);
    }

    #[test]
    fn handles_split_frames() {
        let (mut manager, addr) = test_manager();
        let mut client = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        let bytes = frame(b"split-across-writes");
        client.write_all(&bytes[..3]).unwrap();
        thread::sleep(Duration::from_millis(50));
        manager.poll();
        assert!(manager.connection_mut(1).unwrap().next_packet().is_none());
        client.write_all(&bytes[3..]).unwrap();
        assert_eq!(recv_packet(&mut manager, 1), b"split-across-writes");
    }

    #[test]
    fn writes_reach_the_client() {
        let (mut manager, addr) = test_manager();
        let mut client = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        let bytes = frame(b"from-server");
        manager.connection_mut(1).unwrap().send(&bytes).unwrap();

        let mut header = [0u8; 4];
        client.read_exact(&mut header).unwrap();
        let len = u32::from_be_bytes(header) as usize;
        let mut body = vec![0u8; len];
        client.read_exact(&mut body).unwrap();
        assert_eq!(body, b"from-server");
    }

    #[test]
    fn client_disconnect_removes_connection() {
        let (mut manager, addr) = test_manager();
        let client = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        drop(client);
        wait_until(&mut manager, |m| m.connection_count() == 0);
        let metrics = manager.metrics();
        assert_eq!(metrics.accepted, 1);
        assert_eq!(metrics.disconnected, 1);
    }

    #[test]
    fn remove_drops_single_connection() {
        let (mut manager, addr) = test_manager();
        let one = connect(addr);
        let two = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 2);

        assert!(manager.remove(1));
        assert!(!manager.remove(1));
        assert!(!manager.remove(99));
        assert_eq!(manager.connection_count(), 1);
        assert_eq!(manager.metrics().disconnected, 1);

        drop((one, two));
        wait_until(&mut manager, |m| m.connection_count() == 0);
    }

    #[test]
    fn tracks_multiple_simultaneous_connections() {
        let (mut manager, addr) = test_manager();
        let one = connect(addr);
        let two = connect(addr);
        let three = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 3);

        let mut ids: Vec<_> = manager.connections().map(Connection::id).collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2, 3]);

        drop((one, two, three));
        wait_until(&mut manager, |m| m.connection_count() == 0);
        assert_eq!(manager.metrics().disconnected, 3);
    }

    #[test]
    fn max_connections_leaves_overflow_in_backlog() {
        let (mut manager, addr) = test_manager();
        manager.set_max_connections(Some(1));
        let first = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        let second = connect(addr);
        for _ in 0..10 {
            manager.poll();
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(manager.connection_count(), 1);

        drop(first);
        wait_until(&mut manager, |m| m.connection_count() == 0);
        wait_until(&mut manager, |m| m.connection_count() == 1);
        assert_eq!(manager.metrics().accepted, 2);
        drop(second);
        wait_until(&mut manager, |m| m.connection_count() == 0);
    }

    #[test]
    fn shutdown_closes_active_connections() {
        let (mut manager, addr) = test_manager();
        let mut one = connect(addr);
        let mut two = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 2);

        manager.shutdown();
        assert_eq!(manager.connection_count(), 0);
        assert_eq!(manager.metrics().disconnected, 2);

        let mut buf = [0u8; 1];
        assert_eq!(one.read(&mut buf).unwrap(), 0);
        assert_eq!(two.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn garbage_bytes_do_not_break_the_server() {
        let (mut manager, addr) = test_manager();
        let mut bad = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 1);

        bad.write_all(&[0xFF; 32]).unwrap();
        thread::sleep(Duration::from_millis(50));
        manager.poll();
        assert!(manager.connection_mut(1).unwrap().next_packet().is_none());

        let mut good = connect(addr);
        wait_until(&mut manager, |m| m.connection_count() == 2);
        good.write_all(&frame(b"still-fine")).unwrap();
        assert_eq!(recv_packet(&mut manager, 2), b"still-fine");

        drop(bad);
        wait_until(&mut manager, |m| m.connection_count() == 1);
    }
}
