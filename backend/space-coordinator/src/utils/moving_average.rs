use std::collections::VecDeque;
use std::ops::{AddAssign, Div, SubAssign};

use chrono::{DateTime, TimeDelta, Utc};
use num_traits::Float;

pub struct MovingAverage<V> {
    queue: VecDeque<(DateTime<Utc>, V)>,
    window: TimeDelta,
    sum: V,
}

impl<V: AddAssign + Float + SubAssign> MovingAverage<V> {
    pub fn new(window: TimeDelta) -> Self {
        Self {
            queue: VecDeque::new(),
            window,
            sum: V::zero(),
        }
    }

    pub fn push(&mut self, value: V) -> <V as Div>::Output {
        let now = Utc::now();

        while let Some(front) = self.queue.front()
            && now - front.0 > self.window
        {
            self.sum -= front.1;
            self.queue.pop_front();
        }

        self.queue.push_back((now, value));
        self.sum += value;

        self.average()
    }

    pub fn average(&self) -> <V as Div>::Output {
        if self.queue.is_empty() {
            V::nan()
        } else {
            self.sum / V::from(self.queue.len()).unwrap()
        }
    }
}
