use tracing_subscriber::{fmt, EnvFilter};

use crate::error::{Error, Result};

pub fn init() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
        .map_err(|_| Error::LoggingAlreadyInitialized)
}
