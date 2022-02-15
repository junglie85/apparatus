use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Clock {
    delta: Duration,
    start: Option<Instant>,
}

impl Clock {
    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn elapsed(&self) -> Duration {
        match self.start {
            Some(start) => start.elapsed(),
            None => Duration::from_secs_f32(0.0),
        }
    }

    pub fn tick(&mut self) {
        let end = Instant::now();
        if let Some(start) = self.start {
            self.delta = end - start;
        }
        self.start = Some(end);
    }
}
