use std::io;
use std::net::SocketAddr;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("failed to bind {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: io::Error,
    },

    #[error("network is already bound")]
    AlreadyBound,

    #[error("connection is closed")]
    ConnectionClosed,
}
