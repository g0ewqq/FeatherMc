use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Instant;

use tracing::{error, info};

use crate::console::{self, ConsoleCommand, Event};
use crate::error::{Error, Result};
use crate::java::{JavaHandler, ServerStatus};
use crate::{
    Clock, Config, NetworkManager, RuntimeDirs, Scheduler, SharedState, State, WorldManager,
};

const STATUS_INTERVAL_TICKS: u64 = 200;

pub struct Server {
    config: Config,
    dirs: RuntimeDirs,
    state: SharedState,
    clock: Clock,
    scheduler: Scheduler,
    network: NetworkManager,
    java: JavaHandler,
    worlds: WorldManager,
}

impl Server {
    #[must_use]
    pub fn new(config: Config, dirs: RuntimeDirs) -> Self {
        let status = ServerStatus::from_config(&config);
        Self {
            config,
            dirs,
            state: SharedState::new(State::Created),
            clock: Clock::new(),
            scheduler: Scheduler::new(),
            network: NetworkManager::new(),
            java: JavaHandler::new(status),
            worlds: WorldManager::create_default(),
        }
    }

    #[must_use]
    pub fn state(&self) -> SharedState {
        self.state.clone()
    }

    #[must_use]
    pub fn tick_count(&self) -> u64 {
        self.clock.current()
    }

    pub fn scheduler(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    #[must_use]
    pub fn network(&self) -> &NetworkManager {
        &self.network
    }

    #[must_use]
    pub fn worlds(&self) -> &WorldManager {
        &self.worlds
    }

    pub fn worlds_mut(&mut self) -> &mut WorldManager {
        &mut self.worlds
    }

    pub fn run(self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        install_shutdown_handler(&tx)?;
        let _ = console::spawn(&tx)?;
        self.run_until(rx)
    }

    pub fn run_until(mut self, events: Receiver<Event>) -> Result<()> {
        let mut state = State::Initializing;
        self.state.set(state);
        info!(?state, "Starting {}", crate::version::version_string());
        info!(
            "Loading configuration (server name: \"{}\")",
            self.config.server.name
        );
        info!("Using working directory \"{}\"", self.dirs.base.display());

        info!("Starting network");
        let addr = self.config.network.socket_addr()?;
        self.network.bind(addr)?;
        info!("Listening on {addr}");

        self.scheduler.run_every(STATUS_INTERVAL_TICKS, |tick| {
            info!("Server tick {tick}");
        });

        state = State::Running;
        self.state.set(state);
        info!(?state, "{} is ready", self.config.server.name);

        'run: loop {
            let started = Instant::now();
            let tick = self.clock.advance();
            self.scheduler.tick(tick);
            self.network.poll();
            self.java.pump(&mut self.network, tick);
            self.java.tick_physics(&mut self.network, &self.worlds);
            self.java.pump_chunks(&mut self.network, &mut self.worlds);
            self.java.pump_visibility(&mut self.network);
            self.java.pump_blocks(&mut self.network, &mut self.worlds);
            self.java.pump_inventory(&mut self.network);
            self.clock.record(started.elapsed());

            match events.recv_timeout(self.clock.target().saturating_sub(started.elapsed())) {
                Ok(Event::Shutdown) => break 'run,
                Ok(Event::Command(ConsoleCommand::Stop)) => {
                    info!("Stopping server from console");
                    break 'run;
                }
                Ok(Event::Command(ConsoleCommand::Help)) => {
                    info!("Available commands: stop, help");
                }
                Ok(Event::Command(ConsoleCommand::Unknown(input))) => {
                    info!("Unknown command {input:?}. Type \"help\".");
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    error!("Event channel closed unexpectedly; stopping anyway");
                    break 'run;
                }
            }
        }

        state = State::Stopping;
        self.state.set(state);
        info!(?state, "Stopping {}", crate::version::NAME);
        self.network.shutdown();

        state = State::Stopped;
        self.state.set(state);
        info!(?state, "Stopped");

        Ok(())
    }
}

fn install_shutdown_handler(sender: &Sender<Event>) -> Result<()> {
    let sender = sender.clone();
    ctrlc::set_handler(move || {
        let _ = sender.send(Event::Shutdown);
    })
    .map_err(|e| Error::ShutdownHandler(e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
    use std::time::Duration;

    use super::*;

    fn test_server() -> (tempfile::TempDir, Server) {
        let tmp = tempfile::tempdir().unwrap();
        let config = Config {
            network: crate::config::NetworkConfig {
                address: "127.0.0.1".to_owned(),
                java_port: 0,
            },
            ..Config::default()
        };
        let server = Server::new(config, RuntimeDirs::new(tmp.path()));
        (tmp, server)
    }

    fn wait_for_state(state: &SharedState, want: State) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while state.get() != want {
            assert!(Instant::now() < deadline, "timed out waiting for {want:?}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn transitions_from_created_to_stopped() {
        let (_tmp, server) = test_server();
        assert_eq!(server.state().get(), State::Created);

        let state = server.state();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(|| server.run_until(rx).unwrap());

        wait_for_state(&state, State::Running);
        tx.send(Event::Shutdown).unwrap();
        handle.join().unwrap();

        assert_eq!(state.get(), State::Stopped);
    }

    #[test]
    fn stops_from_console_command() {
        let (_tmp, server) = test_server();
        let state = server.state();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(|| server.run_until(rx).unwrap());

        wait_for_state(&state, State::Running);
        tx.send(Event::Command(ConsoleCommand::Stop)).unwrap();
        handle.join().unwrap();

        assert_eq!(state.get(), State::Stopped);
    }

    #[test]
    fn ticks_advance_and_tasks_run_while_running() {
        let (_tmp, mut server) = test_server();
        let count = Arc::new(AtomicU64::new(0));
        let task_count = count.clone();
        server.scheduler().run_every(
            1,
            Box::new(move |_| {
                task_count.fetch_add(1, Ordering::SeqCst);
            }),
        );

        let state = server.state();
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(|| server.run_until(rx).unwrap());

        wait_for_state(&state, State::Running);
        std::thread::sleep(Duration::from_millis(300));
        tx.send(Event::Shutdown).unwrap();
        handle.join().unwrap();

        assert!(count.load(Ordering::SeqCst) >= 2);
        assert_eq!(state.get(), State::Stopped);
    }
}
