use chrono::DateTime;
use kalshi_rs::websocket::models::OrderbookDeltaMessage;
use serde::Deserialize;

use super::tick::Tick;

#[derive(Deserialize, Debug)]
pub struct KalshiOrderbookUpdate {
    pub ts: u64, // unix time stamp
    pub price: u64, // price in hundreths of a cent
    pub delta: i64, // change in contract amount at price level
}

impl KalshiOrderbookUpdate {
    pub fn new(ts: u64, price: u64, delta: i64) -> Self {
        KalshiOrderbookUpdate { ts, price, delta }
    }
}

impl Tick for KalshiOrderbookUpdate {
    fn ts(&self) -> u64 {
        self.ts
    }
}

impl TryFrom<OrderbookDeltaMessage> for KalshiOrderbookUpdate {
    type Error = Box<dyn std::error::Error>;

    fn try_from(message: OrderbookDeltaMessage) -> Result<Self, Self::Error> {
        let ts = DateTime::parse_from_rfc3339(
            &message.ts
        )?.timestamp() as u64;
        let price = message.price as u64;
        let delta = message.delta;
        Ok(KalshiOrderbookUpdate { ts, price, delta })
    }
}