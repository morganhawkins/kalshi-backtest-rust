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
    ts: u64, // unix time stamp
    price: f64, // open price
    mu: f64, // mean log return _h rolling window
    sigma: f64, // sigma log return _h rolling window
}

