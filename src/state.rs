use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Created = 0,
    Initializing = 1,
    Running = 2,
    Stopping = 3,
    Stopped = 4,
}

#[derive(Debug, Clone)]
pub struct SharedState {
    inner: Arc<AtomicU8>,
}

impl SharedState {
    #[must_use]
    pub fn new(state: State) -> Self {
        Self {
            inner: Arc::new(AtomicU8::new(state as u8)),
        }
    }

    #[must_use]
    pub fn get(&self) -> State {
        match self.inner.load(Ordering::SeqCst) {
            0 => State::Created,
            1 => State::Initializing,
            2 => State::Running,
            3 => State::Stopping,
            _ => State::Stopped,
        }
    }

    pub fn set(&self, state: State) {
        self.inner.store(state as u8, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_every_variant() {
        let state = SharedState::new(State::Created);
        for next in [
            State::Created,
            State::Initializing,
            State::Running,
            State::Stopping,
            State::Stopped,
        ] {
            state.set(next);
            assert_eq!(state.get(), next);
        }
    }

    #[test]
    fn clones_observe_the_same_state() {
        let state = SharedState::new(State::Created);
        let other = state.clone();
        state.set(State::Running);
        assert_eq!(other.get(), State::Running);
    }
}
