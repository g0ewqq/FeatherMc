#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("failed to serialize configuration: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("logging is already initialized")]
    LoggingAlreadyInitialized,

    #[error("failed to install shutdown handler: {0}")]
    ShutdownHandler(String),
}

pub type Result<T> = std::result::Result<T, Error>;
