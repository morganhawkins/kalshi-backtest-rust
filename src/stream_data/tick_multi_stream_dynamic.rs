use std::rc::Rc;

use super::tick_stream::DataStream;
use super::timers::Timer;
use crate::data_models::tick::Tick;

/// A collection of data streams that share a common timer.
/// Uses dynamic dispatch to allow streams of different data types.
pub struct TickMultiStream<'a, T: Timer> {
    timer: Rc<T>,
    streams: Vec<Box<dyn DataStream + 'a>>,
}

impl<'a, T: Timer> TickMultiStream<'a, T> {
    pub fn new(timer: Rc<T>) -> Self {
        TickMultiStream {
            timer,
            streams: Vec::new(),
        }
    }

    // TODO: Need to enforce that the timers are the same to ensure
    // temporal sync
    /// Adds a stream to the collection. Returns the stream index.
    pub fn add_stream(&mut self, stream: Box<dyn DataStream + 'a>) -> usize {
        let idx = self.streams.len();
        self.streams.push(stream);
        idx
    }

    /// Returns an iterator over (stream_index, items) for all streams.
    /// Each stream's new items are collected together. When a stream has concluded,
    /// it will no longer appear
    ///
    /// Returns None if the timer has expired.
    pub fn iter_all_new(&self) -> Option<Vec<(usize, Vec<&dyn Tick>)>> {
        // If simulation has concluded, return None
        self.timer.get_time()?;

        let results: Vec<_> = self
            .streams 
            .iter() // iter through streams
            .enumerate() // enumerate to preserve index
            .filter_map(|(idx, stream)| stream.iter_new().map(|items| (idx, items))) // filter out indices with no updates remaining in their stream
            .collect();

        Some(results)
    }

    /// Returns new items from a specific stream.
    ///
    /// Returns None if timer expired or stream index is out of bounds.
    pub fn iter_new_for_stream(&self, stream_idx: usize) -> Option<Vec<&dyn Tick>> {
        self.streams.get(stream_idx)?.iter_new()
    }

    /// Advances the timer by one unit
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

    /// Returns the number of streams.
    pub fn num_streams(&self) -> usize {
        self.streams.len()
    }

    /// Returns the current index for a specific stream.
    pub fn current_index(&self, stream_idx: usize) -> Option<usize> {
        self.streams.get(stream_idx).map(|s| s.current_index())
    }

    /// Returns true if all streams are exhausted.
    pub fn all_exhausted(&self) -> bool {
        self.streams.iter().all(|s| s.is_exhausted())
    }

    /// Returns true if all streams are exhausted.
    pub fn any_exhausted(&self) -> bool {
        self.streams.iter().any(|s| s.is_exhausted())
    }

    /// Returns true if a specific stream is exhausted.
    pub fn is_stream_exhausted(&self, stream_idx: usize) -> bool {
        self.streams
            .get(stream_idx)
            .map(|s| s.is_exhausted())
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_data::tick_stream::TickStream;
    use crate::stream_data::timers::DeltaTimer;

    struct MockTickA {
        ts: u64,
        _value: i32,
    }

    impl MockTickA {
        fn new(ts: u64, _value: i32) -> Self {
            MockTickA { ts, _value }
        }
    }

    impl Tick for MockTickA {
        fn ts(&self) -> u64 {
            self.ts
        }
    }

    struct MockTickB {
        ts: u64,
        _label: &'static str,
    }

    impl MockTickB {
        fn new(ts: u64, _label: &'static str) -> Self {
            MockTickB { ts, _label }
        }
    }

    impl Tick for MockTickB {
        fn ts(&self) -> u64 {
            self.ts
        }
    }

    #[test]
    fn test_heterogeneous_streams() {
        let timer = Rc::new(DeltaTimer::new(5, 20, 5));

        let data_a = vec![
            MockTickA::new(1, 100),
            MockTickA::new(5, 200),
            MockTickA::new(10, 300),
        ];
        let data_b = vec![
            MockTickB::new(2, "first"),
            MockTickB::new(4, "second"),
            MockTickB::new(8, "third"),
        ];

        let stream_a = TickStream::new(Rc::clone(&timer), &data_a);
        let stream_b = TickStream::new(Rc::clone(&timer), &data_b);

        let mut multi = TickMultiStream::new(Rc::clone(&timer));
        multi.add_stream(Box::new(stream_a));
        multi.add_stream(Box::new(stream_b));

        // At time 5
        let results = multi.iter_all_new().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1.len(), 2); // stream_a: ts 1, 5
        assert_eq!(results[1].1.len(), 2); // stream_b: ts 2, 4

        // Cycle to time 10
        multi.cycle();
        let results = multi.iter_all_new().unwrap();
        assert_eq!(results[0].1.len(), 1); // stream_a: ts 10
        assert_eq!(results[1].1.len(), 1); // stream_b: ts 8
    }

    #[test]
    fn test_shared_timer_across_multi_streams() {
        let timer = Rc::new(DeltaTimer::new(5, 20, 5));

        let data1 = vec![MockTickA::new(1, 1)];
        let data2 = vec![MockTickA::new(2, 2)];

        let stream1 = TickStream::new(Rc::clone(&timer), &data1);
        let stream2 = TickStream::new(Rc::clone(&timer), &data2);

        let mut multi1 = TickMultiStream::new(Rc::clone(&timer));
        multi1.add_stream(Box::new(stream1));

        let mut multi2 = TickMultiStream::new(Rc::clone(&timer));
        multi2.add_stream(Box::new(stream2));

        assert_eq!(multi1.get_time(), Some(5));
        assert_eq!(multi2.get_time(), Some(5));

        // Cycle via one multi stream
        multi1.cycle();

        // Both see updated time
        assert_eq!(multi1.get_time(), Some(10));
        assert_eq!(multi2.get_time(), Some(10));
    }

    #[test]
    fn test_iter_new_for_stream() {
        let timer = Rc::new(DeltaTimer::new(5, 20, 5));
        let data = vec![MockTickA::new(1, 1), MockTickA::new(5, 2)];

        let stream = TickStream::new(Rc::clone(&timer), &data);

        let mut multi = TickMultiStream::new(timer);
        multi.add_stream(Box::new(stream));

        let items = multi.iter_new_for_stream(0).unwrap();
        assert_eq!(items.len(), 2);

        // Out of bounds returns None
        assert!(multi.iter_new_for_stream(1).is_none());
    }

    #[test]
    fn test_is_exhausted() {
        let timer = Rc::new(DeltaTimer::new(0, 10, 5));
        let data = vec![MockTickA::new(0, 1)];

        let stream = TickStream::new(Rc::clone(&timer), &data);

        let mut multi = TickMultiStream::new(timer);
        multi.add_stream(Box::new(stream));

        assert!(!multi.all_exhausted());
        let _ = multi.iter_all_new();
        assert!(multi.all_exhausted());
    }
}
