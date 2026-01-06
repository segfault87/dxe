use std::collections::VecDeque;

use chrono::{DateTime, TimeDelta, Utc};

pub struct MovingAverage {
    queue: VecDeque<(DateTime<Utc>, f64)>,
    window: TimeDelta,
    sum: f64,
}

impl MovingAverage {
    pub fn new(window: TimeDelta) -> Self {
        Self {
            queue: VecDeque::new(),
            window,
            sum: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) -> f64 {
        let now = Utc::now();

        while let Some(front) = self.queue.front()
            && now - front.0 < self.window
        {
            self.sum -= front.1;
            self.queue.pop_front();
        }

        self.queue.push_back((now, value));
        self.sum += value;

        self.average()
    }

    pub fn average(&self) -> f64 {
        if self.queue.is_empty() {
            f64::NAN
        } else {
            self.sum / self.queue.len() as f64
        }
    }
}
