use std::rc::Rc;

use super::tick_stream::TickStream;
use super::timers::Timer;
use crate::data_models::tick::Tick;

/// Macro to define a multi source (but fixed number of) streasm of arbitrary types.
/// Trying to avoid dynamic dispatch by generating a struct with N-many generic type fields
/// IDK if this is the right move yet, but I believe this would be more performant
///
#[macro_export]
macro_rules! define_multi_stream {
    ($name:ident, 1) => {
        $crate::define_multi_stream_impl!($name, (S0, D0, 0));
    };
    ($name:ident, 2) => {
        $crate::define_multi_stream_impl!($name, (S0, D0, 0), (S1, D1, 1));
    };
    ($name:ident, 3) => {
        $crate::define_multi_stream_impl!($name, (S0, D0, 0), (S1, D1, 1), (S2, D2, 2));
    };
    ($name:ident, 4) => {
        $crate::define_multi_stream_impl!($name, (S0, D0, 0), (S1, D1, 1), (S2, D2, 2), (S3, D3, 3));
    };
    ($name:ident, 5) => {
        $crate::define_multi_stream_impl!($name, (S0, D0, 0), (S1, D1, 1), (S2, D2, 2), (S3, D3, 3), (S4, D4, 4));
    };
    ($name:ident, 6) => {
        $crate::define_multi_stream_impl!($name, (S0, D0, 0), (S1, D1, 1), (S2, D2, 2), (S3, D3, 3), (S4, D4, 4), (S5, D5, 5));
    };
}

#[macro_export]
macro_rules! define_multi_stream_impl {
    ($name:ident, $(($timer_ty:ident, $data_ty:ident, $idx:tt)),+) => {
        pub struct $name<'a, T: Timer, $($data_ty: Tick),+> {
            timer: Rc<T>,
            streams: ($(TickStream<'a, T, $data_ty>),+),
        }

        impl<'a, T: Timer, $($data_ty: Tick),+> $name<'a, T, $($data_ty),+> {
            pub fn new(
                timer: Rc<T>,
                $([<stream_ $idx>]: TickStream<'a, T, $data_ty>),+
            ) -> Self {
                Self {
                    timer,
                    streams: ($([<stream_ $idx>]),+),
                }
            }

            /// Returns an iterator over (stream_index, items) for all streams.
            pub fn iter_all_new(&self) -> Option<Vec<(usize, Vec<&dyn Tick>)>> {
                self.timer.get_time()?;

                let mut results = Vec::new();
                $(
                    if let Some(items) = self.streams.$idx.iter_new() {
                        if !items.is_empty() {
                            results.push(($idx, items));
                        }
                    }
                )+
                Some(results)
            }

            /// Advances timer by one unit.
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

            /// Returns the number of streams (compile-time constant).
            pub const fn num_streams(&self) -> usize {
                $crate::count!($($idx),+)
            }

            /// Returns true if all streams are exhausted.
            pub fn all_exhausted(&self) -> bool {
                true $(&& self.streams.$idx.is_exhausted())+
            }

            /// Returns true if any stream is exhausted.
            pub fn any_exhausted(&self) -> bool {
                false $(|| self.streams.$idx.is_exhausted())+
            }
        }
    };
}

/// Helper macro to count arguments.
#[macro_export]
macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)*) => (1usize + $crate::count!($($xs)*));
}