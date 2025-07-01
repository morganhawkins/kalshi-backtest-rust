use std::cell::RefCell;

pub trait Timer {
    fn cycle(&self);
    fn get_time(&self) -> Option<u64>;
}

pub struct DeltaTimer {
    start_ts: u64,
    end_ts: u64,
    current_ts: RefCell<u64>,
    delta: u64,
}

impl DeltaTimer {
    // current_ts init to same as start_ts
    pub fn new(start_ts: u64, end_ts: u64, delta: u64) -> Self {
        DeltaTimer {
            start_ts: start_ts,
            end_ts: end_ts,
            current_ts: RefCell::new(start_ts),
            delta: delta,
        }
    }
}

impl Timer for DeltaTimer {
    // jumps current_ts by delta
    fn cycle(&self) {
        let mut ts = self.current_ts.borrow_mut();
        *ts += self.delta;
    }

    // returns Some(timestamp) if timer is still valid
    fn get_time(&self) -> Option<u64> {
        if *self.current_ts.borrow() <= self.end_ts {
            Some(*self.current_ts.borrow())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_timer() {
        let timer = DeltaTimer::new(0, 10, 1);
        timer.cycle();
        assert!(timer.get_time() == Some(1u64));
    }
}
