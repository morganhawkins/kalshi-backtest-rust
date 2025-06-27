use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct KalshiRecord {
    ts: u64,
    tte: f64,
    bid: u8,
    ask: u8,
}

#[derive(Deserialize, Debug)]
pub struct KalshiDataset {
    pub records: Vec<KalshiRecord>
}