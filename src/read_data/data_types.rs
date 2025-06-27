use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KalshiRecord {
    ts: u64,
    tte: f64,
    bid: u8,
    ask: u8,
}

#[derive(Deserialize, Debug)]
pub struct CoinbaseRecord {
    ts: u64,
    price: f64,
    volume: f64,
    mu: f64,
    sigma: f64,
}

