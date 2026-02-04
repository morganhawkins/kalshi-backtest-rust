use serde::Deserialize;
use super::tick::Tick;

#[derive(Deserialize, Debug)]
pub struct CoinbaseRecord {
    pub ts: u64, // unix time stamp
    pub price: f64, // open price for minute interval
}

impl CoinbaseRecord {
    pub fn new(ts: u64, price: f64) -> Self {
        Self {
            ts,
            price,
        }
    }
}

impl Tick for CoinbaseRecord {
    fn ts(&self) -> u64 {
        self.ts
    }
}
