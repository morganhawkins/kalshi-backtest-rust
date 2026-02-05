use std::cell::Cell;
use std::rc::Rc;

use super::timers::Timer;
use crate::data_models::tick::Tick;

/// Trait for a data stream that yields tick data based on a timer.
/// Used for dynamic dispatch in TickMultiStream.
pub trait DataStream {
    /// Returns new datapoints (not yet iterated, ts <= current time) as trait objects.
    /// Returns None if the timer has expired.
    fn iter_new(&self) -> Option<Vec<&dyn Tick>>;

    /// Returns the current index position in the data.
    fn current_index(&self) -> usize;

    /// Returns true if all data has been consumed.
    fn is_exhausted(&self) -> bool;

    /// Returns the length of the underlying data.
    fn len(&self) -> usize;
}

/// A stream that yields datapoints as they become "ready" based on a Timer.
/// Tracks which datapoints have been yielded and only returns new ones
/// whose timestamp is <= the current time.
pub struct TickStream<'a, T: Timer, D: Tick> {
    timer: Rc<T>,
    data: &'a [D],
    current_index: Cell<usize>,
}

impl<'a, T: Timer, D: Tick> TickStream<'a, T, D> {
    pub fn new(timer: Rc<T>, data: &'a [D]) -> Self {
        TickStream {
            timer,
            data,
            current_index: Cell::new(0),
        }
    }

    /// Returns an iterator over datapoints that:
    /// - Have not been iterated before
    /// - Have a timestamp <= the current time from the Timer
    ///
    /// Returns None if the timer has expired (get_time returns None).
    pub fn iter_new_typed(&self) -> Option<impl Iterator<Item = &D>> {
        let current_time = self.timer.get_time()?;
        let start_index = self.current_index.get();

        let mut end_index = start_index;
        while end_index < self.data.len() && self.data[end_index].ts() <= current_time {
            end_index += 1;
        }

        self.current_index.set(end_index);

        Some(self.data[start_index..end_index].iter())
    }

    /// Advances the timer by one cycle.
    pub fn cycle(&self) {
        self.timer.cycle();
    }

    /// Returns the current time from the timer, if valid.
    pub fn get_time(&self) -> Option<u64> {
        self.timer.get_time()
    }

    /// Returns a reference to the shared timer.
    pub fn timer(&self) -> &Rc<T> {
        &self.timer
    }
}

impl<'a, T: Timer, D: Tick> DataStream for TickStream<'a, T, D> {
    fn iter_new(&self) -> Option<Vec<&dyn Tick>> {
        let current_time = self.timer.get_time()?;
        let start_index = self.current_index.get();

        let mut end_index = start_index;
        while end_index < self.data.len() && self.data[end_index].ts() <= current_time {
            end_index += 1;
        }

        self.current_index.set(end_index);

        let items: Vec<&dyn Tick> = self.data[start_index..end_index]
            .iter()
            .map(|d| d as &dyn Tick)
            .collect();

        Some(items)
    }

    fn current_index(&self) -> usize {
        self.current_index.get()
    }

    fn is_exhausted(&self) -> bool {
        self.current_index.get() >= self.data.len()
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_data::timers::DeltaTimer;

    struct MockTick {
        ts: u64,
        value: i32,
    }

    impl MockTick {
        fn new(ts: u64, value: i32) -> Self {
            MockTick { ts, value }
        }
    }

    impl Tick for MockTick {
        fn ts(&self) -> u64 {
            self.ts
        }
    }

    #[test]
    fn test_iter_new_typed_returns_ready_items() {
        let timer = Rc::new(DeltaTimer::new(5, 20, 5));
        let data = vec![
            MockTick::new(1, 1),
            MockTick::new(3, 2),
            MockTick::new(5, 3),
            MockTick::new(7, 4),
            MockTick::new(10, 5),
        ];

        let stream = TickStream::new(timer, &data);

        // At time 5, items with ts <= 5 should be ready (indices 0, 1, 2)
        let items: Vec<_> = stream.iter_new_typed().unwrap().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].value, 1);
        assert_eq!(items[1].value, 2);
        assert_eq!(items[2].value, 3);

        // Calling iter_new again without cycling should return empty
        let items: Vec<_> = stream.iter_new_typed().unwrap().collect();
        assert_eq!(items.len(), 0);

        // Cycle to time 10
        stream.cycle();
        let items: Vec<_> = stream.iter_new_typed().unwrap().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].value, 4);
        assert_eq!(items[1].value, 5);
    }

    #[test]
    fn test_data_stream_trait() {
        let timer = Rc::new(DeltaTimer::new(5, 20, 5));
        let data = vec![MockTick::new(1, 1), MockTick::new(5, 2)];

        let stream = TickStream::new(timer, &data);

        // Use via DataStream trait
        let boxed: Box<dyn DataStream> = Box::new(stream);

        assert!(!boxed.is_exhausted());
        assert_eq!(boxed.len(), 2);

        let items = boxed.iter_new().unwrap();
        assert_eq!(items.len(), 2);

        assert!(boxed.is_exhausted());
    }

    #[test]
    fn test_timer_expiration_returns_none() {
        let timer = Rc::new(DeltaTimer::new(0, 2, 5));
        let data = vec![MockTick::new(1, 1)];

        let stream = TickStream::new(timer, &data);

        assert!(stream.iter_new().is_some());

        stream.cycle();
        assert!(stream.iter_new().is_none());
    }

    #[test]
    fn test_is_exhausted() {
        let timer = Rc::new(DeltaTimer::new(0, 10, 5));
        let data = vec![MockTick::new(0, 1), MockTick::new(1, 2)];

        let stream = TickStream::new(timer, &data);
        assert!(!stream.is_exhausted());

        stream.cycle();

        let _: Vec<_> = stream.iter_new_typed().unwrap().collect();
        assert!(stream.is_exhausted());
    }
}
