use serde::Deserialize;

use crate::read_data::read_underlying;

#[derive(Deserialize, Debug)]
pub struct KalshiRecord {
    pub ts: u64,
    pub tte: f64,
    pub bid: u8,
    pub ask: u8,
}

impl KalshiRecord {
    pub fn new(ts: u64, tte: f64, bid: u8, ask: u8) -> Self{
        KalshiRecord{ts, tte, bid, ask}
    }
}

pub struct KalshiDataset {
    pub records: Vec<KalshiRecord>, // vector of timestamp data
    pub strike: f64, // contract strike price
    pub start_ts: u64, // unix timestamp - time that the dataset starts collecting data
    pub expir_ts: u64, // unix timestamp - time that the contract expires
}

impl KalshiDataset {
    pub fn new(records: Vec<KalshiRecord>, strike: f64, start_ts: u64, expir_ts: u64) -> Self {
        KalshiDataset { records, strike, start_ts, expir_ts }
    }

    pub fn relevant_slice<'a>(&self, underlying_data: &'a Vec<CoinbaseRecord>) -> &'a [CoinbaseRecord]{

        return &underlying_data[1..3];
    }

}


#[derive(Deserialize, Debug)]
pub struct CoinbaseRecord {
    pub ts: u64, // unix time stamp
    pub price: f64, // open price
    pub mu: f64, // mean log return _h rolling window
    pub sigma: f64, // sigma log return _h rolling window
}

