use std::time::Duration;

pub const TICKS_PER_SECOND: u32 = 20;
pub const TICK_DURATION: Duration = Duration::from_millis(1000 / TICKS_PER_SECOND as u64);

#[derive(Debug)]
pub struct Clock {
    tick: u64,
    target: Duration,
    last_duration: Duration,
}

impl Clock {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick: 0,
            target: TICK_DURATION,
            last_duration: Duration::ZERO,
        }
    }

    #[must_use]
    pub fn current(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub fn target(&self) -> Duration {
        self.target
    }

    #[must_use]
    pub fn last_duration(&self) -> Duration {
        self.last_duration
    }

    pub fn advance(&mut self) -> u64 {
        let tick = self.tick;
        self.tick += 1;
        tick
    }

    pub fn record(&mut self, duration: Duration) {
        self.last_duration = duration;
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero_and_counts_up() {
        let mut clock = Clock::new();
        assert_eq!(clock.current(), 0);
        assert_eq!(clock.advance(), 0);
        assert_eq!(clock.advance(), 1);
        assert_eq!(clock.current(), 2);
    }

    #[test]
    fn targets_twenty_ticks_per_second() {
        let clock = Clock::new();
        assert_eq!(TICKS_PER_SECOND, 20);
        assert_eq!(clock.target(), Duration::from_millis(50));
    }

    #[test]
    fn records_last_tick_duration() {
        let mut clock = Clock::new();
        assert_eq!(clock.last_duration(), Duration::ZERO);
        clock.record(Duration::from_millis(7));
        assert_eq!(clock.last_duration(), Duration::from_millis(7));
    }
}
