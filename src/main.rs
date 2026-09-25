use std::process::ExitCode;

use tracing::error;

use spironmc::{Config, RuntimeDirs, Server};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> spironmc::Result<()> {
    if let Err(err) = spironmc::logging::init() {
        if !matches!(err, spironmc::Error::LoggingAlreadyInitialized) {
            return Err(err);
        }
    }

    let base = std::env::current_dir()?;
    let dirs = RuntimeDirs::new(&base);
    dirs.ensure_all()?;

    let config_path = dirs.config_file();
    let config = Config::load_or_create(&config_path)?;

    Server::new(config, dirs).run()
}
