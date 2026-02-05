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

