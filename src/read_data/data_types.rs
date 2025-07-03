use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KalshiRecord {
    pub ts: u64,
    pub tte: f64,
    pub bid: u8,
    pub ask: u8,
}

impl KalshiRecord {
    pub fn new(ts: u64, tte: f64, bid: u8, ask: u8) -> Self {
        KalshiRecord { ts, tte, bid, ask }
    }
}

pub struct KalshiDataset {
    pub records: Vec<KalshiRecord>, // vector of timestamp data
    pub strike: f64,                // contract strike price
    pub start_ts: u64,              // unix timestamp - time that the dataset starts collecting data
    pub expir_ts: u64,              // unix timestamp - time that the contract expires
}

impl KalshiDataset {
    pub fn new(records: Vec<KalshiRecord>, strike: f64, start_ts: u64, expir_ts: u64) -> Self {
        KalshiDataset {
            records,
            strike,
            start_ts,
            expir_ts,
        }
    }

    // finds the CoinbaseRecord with the largest index s.t. it's ts field is still <= `timestamp`
    pub fn find_idx(timestamp: u64, underlying_data: &Vec<CoinbaseRecord>) -> Option<usize> {
        // boundary check - if timestamp if < first element or >= last element,
        //  we cannot find a satisfying element. Otherwise we can definitely find an element
        {
            let first_record = match underlying_data.get(0) {
                Some(val) => val,
                None => return None,
            };
            let last_record = match underlying_data.last() {
                Some(val) => val,
                None => return None,
            };

            if timestamp < first_record.ts || timestamp >= last_record.ts {
                return None;
            }
        }

        // binary search view start idx, view end idx, and current idx
        let mut range_start = 0_usize;
        let mut range_end = underlying_data.len();
        let mut idx = underlying_data.len() / 2_usize;

        let mut idx_ts: u64;
        let mut next_idx_ts: u64;
        loop {
            // current idx timestamp and next item timestamp update
            idx_ts = underlying_data[idx].ts;
            next_idx_ts = underlying_data[idx + 1].ts;

            // check if search condition satisfied
            if idx_ts <= timestamp && next_idx_ts > timestamp {
                return Some(idx);
            }

            // change search range to upper half or lower half
            if idx_ts <= timestamp {
                // if idx ts is still lower than `timestamp` -> look at upper half
                range_start = idx;
            } else {
                // if idx ts is too high -> look at lower half
                range_end = idx;
            }
            // update idx to be in middle of range
            idx = (range_start + range_end) / 2_usize;
        }
    }

    // NOTE: assuming that vector is sorted on `ts` field of CoinbaseRecord
    pub fn relevant_slice<'a>(
        &self,
        underlying_data: &'a Vec<CoinbaseRecord>,
    ) -> Option<&'a [CoinbaseRecord]> {
        let slice_start = Self::find_idx(self.start_ts, underlying_data)?;
        let slice_end = Self::find_idx(self.expir_ts, underlying_data)?;

        // panic in case of unexpected behavior
        if slice_end <= slice_start {
            panic!("Data collection issue, underlying data is unsorted.");
        }

        // can safely add 1 to end_slice since it will never be last element
        return Some(&underlying_data[slice_start..slice_end + 1usize]);
    }
}

#[derive(Deserialize, Debug)]
pub struct CoinbaseRecord {
    pub ts: u64,    // unix time stamp
    pub price: f64, // open price
    pub mu: f64,    // mean log return _h rolling window
    pub sigma: f64, // sigma log return _h rolling window
}

impl CoinbaseRecord {
    pub fn new(ts: u64, price: f64, mu: f64, sigma: f64) -> Self {
        Self {
            ts,
            price,
            mu,
            sigma,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_cb_records(ts_start: u64, ts_end: u64) -> Vec<CoinbaseRecord> {
        let mut mock = Vec::new();
        for ts in ts_start..ts_end {
            mock.push(CoinbaseRecord::new(ts * 2_u64, 0.0, 0.0, 0.0));
        }
        mock
    }

    #[test]
    fn test_binary_underlying_records() {
        let mock_cb_records = create_mock_cb_records(2, 8);

        // test non-exact match
        let start_record_idx = KalshiDataset::find_idx(7, &mock_cb_records);
        assert!(start_record_idx.unwrap() == 1usize);
        // test exact match
        let start_record_idx = KalshiDataset::find_idx(8, &mock_cb_records);
        assert!(start_record_idx.unwrap() == 2usize);
        // test out-of-bounds upper
        let start_record_idx = KalshiDataset::find_idx(20, &mock_cb_records);
        assert!(start_record_idx.is_none());
        // test out-of-bounds lower
        let start_record_idx = KalshiDataset::find_idx(0, &mock_cb_records);
        assert!(start_record_idx.is_none());
    }
}
