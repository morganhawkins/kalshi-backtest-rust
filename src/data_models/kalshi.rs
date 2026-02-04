use serde::Deserialize;
use super::tick::Tick;

#[derive(Deserialize, Debug)]
pub struct KalshiRecord {
    pub ts: u64, // unix time stamp
    pub tte: f64, // hours until contract expiration
    pub bid: u32, // bid price, in hundreths of a cent
    pub ask: u32, // ask price, in hundreths of a cent
}

impl KalshiRecord {
    pub fn new(ts: u64, tte: f64, bid: u32, ask: u32) -> Self {
        KalshiRecord { ts, tte, bid, ask }
    }
}

impl Tick for KalshiRecord {
    fn ts(&self) -> u64 {
        self.ts
    }
}