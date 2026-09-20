pub type Task = Box<dyn FnMut(u64) + Send>;

struct OnceTask {
    run_at: u64,
    task: Task,
}

struct RepeatingTask {
    next: u64,
    period: u64,
    task: Task,
}

#[derive(Default)]
pub struct Scheduler {
    now: u64,
    once: Vec<OnceTask>,
    repeating: Vec<RepeatingTask>,
}

impl Scheduler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run_next_tick(&mut self, task: impl FnMut(u64) + Send + 'static) {
        self.run_later(0, task);
    }

    pub fn run_later(&mut self, delay: u64, task: impl FnMut(u64) + Send + 'static) {
        self.once.push(OnceTask {
            run_at: self.now.saturating_add(delay),
            task: Box::new(task),
        });
    }

    pub fn run_every(&mut self, period: u64, task: impl FnMut(u64) + Send + 'static) {
        assert!(period > 0, "scheduler period must be at least 1 tick");
        self.repeating.push(RepeatingTask {
            next: self.now.saturating_add(period),
            period,
            task: Box::new(task),
        });
    }

    pub fn tick(&mut self, now: u64) {
        self.now = now;

        let mut i = 0;
        while i < self.once.len() {
            if self.once[i].run_at <= now {
                let mut entry = self.once.remove(i);
                (entry.task)(now);
            } else {
                i += 1;
            }
        }

        for task in &mut self.repeating {
            if now >= task.next {
                (task.task)(now);
                task.next = now.saturating_add(task.period);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    use super::*;

    fn counter() -> (Arc<AtomicU64>, Task) {
        let count = Arc::new(AtomicU64::new(0));
        let task_count = count.clone();
        let task: Task = Box::new(move |_| {
            task_count.fetch_add(1, Ordering::SeqCst);
        });
        (count, task)
    }

    #[test]
    fn runs_next_tick_task_once() {
        let mut scheduler = Scheduler::new();
        let (count, task) = counter();
        scheduler.run_next_tick(task);

        scheduler.tick(0);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        scheduler.tick(1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn runs_delayed_task_at_the_right_tick() {
        let mut scheduler = Scheduler::new();
        let (count, task) = counter();
        scheduler.run_later(5, task);

        for tick in 0..5 {
            scheduler.tick(tick);
            assert_eq!(count.load(Ordering::SeqCst), 0);
        }
        scheduler.tick(5);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn runs_repeating_task_every_period() {
        let mut scheduler = Scheduler::new();
        let (count, task) = counter();
        scheduler.run_every(3, task);

        for tick in 0..10 {
            scheduler.tick(tick);
        }
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn runs_tasks_in_schedule_order() {
        let mut scheduler = Scheduler::new();
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        for id in [1, 2, 3] {
            let order = order.clone();
            scheduler.run_next_tick(Box::new(move |_| order.lock().unwrap().push(id)));
        }

        scheduler.tick(0);
        assert_eq!(*order.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn passes_current_tick_to_tasks() {
        let mut scheduler = Scheduler::new();
        let seen = Arc::new(AtomicU64::new(u64::MAX));
        let task_seen = seen.clone();
        scheduler.run_later(
            4,
            Box::new(move |tick| task_seen.store(tick, Ordering::SeqCst)),
        );

        scheduler.tick(4);
        assert_eq!(seen.load(Ordering::SeqCst), 4);
    }

    #[test]
    #[should_panic(expected = "scheduler period must be at least 1 tick")]
    fn rejects_zero_period() {
        Scheduler::new().run_every(0, Box::new(|_| {}));
    }
}
