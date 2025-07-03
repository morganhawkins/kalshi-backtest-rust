use std::cell::RefCell;

use super::timers::{DeltaTimer, Timer};
use crate::read_data::data_types::{CoinbaseRecord, KalshiDataset, KalshiRecord};

// Pairing a slice reference of underlying data along with owned derivative data
// To test hedging of a single contract
pub struct PairMarketStream<'a> {
    pub under_data: &'a [CoinbaseRecord],
    pub deriv_data: KalshiDataset,
    timer: DeltaTimer,
    pub under_current_index: Option<RefCell<usize>>,
    pub deriv_current_index: Option<RefCell<usize>>,
}

impl<'a> PairMarketStream<'a> {
    pub fn new(under_data: &'a [CoinbaseRecord], deriv_data: KalshiDataset, delta: u64) -> Self {
        let timer = DeltaTimer::new(deriv_data.start_ts, deriv_data.expir_ts, delta);

        let stream = PairMarketStream {
            under_data: under_data,
            deriv_data: deriv_data,
            timer: timer,
            under_current_index: Some(RefCell::new(0)),
            deriv_current_index: Some(RefCell::new(0)),
        };
        stream.update_indices();

        stream
    }

    /// Returns some if a next index was found, else returns None
    pub fn update_indices(&self) -> Option<()> {
        let curr_time = self.timer.get_time()?;

        if self.under_current_index.is_none() || self.deriv_current_index.is_none() {
            return None;
        }

        // Underlying index update
        let search_start = *self.under_current_index.as_ref().unwrap().borrow();
        let mut new_under_index: Option<usize> = None;
        for i in search_start..self.under_data.len() - 1 {
            if self.under_data[i].ts <= curr_time && self.under_data[i + 1].ts > curr_time {
                new_under_index = Some(i);
                break;
            }
        }

        // Dertivative index update
        let search_start = *self.deriv_current_index.as_ref().unwrap().borrow();
        let mut new_deriv_index: Option<usize> = None;
        for i in search_start..self.deriv_data.records.len() - 1 {
            if self.deriv_data.records[i].ts <= curr_time
                && self.deriv_data.records[i + 1].ts > curr_time
            {
                new_deriv_index = Some(i);
                break;
            }
        }

        // if new index found for both derivative and underlying return Some
        match (new_under_index, new_deriv_index) {
            (Some(under_index), Some(deriv_index)) => {
                *self.under_current_index.as_ref().unwrap().borrow_mut() = under_index;
                *self.deriv_current_index.as_ref().unwrap().borrow_mut() = deriv_index;
                return Some(());
            }
            _ => return None,
        };
    }

    pub fn cycle(&self) {
        self.timer.cycle();
        self.update_indices();
    }

    pub fn get_time(&self) -> Option<u64> {
        self.timer.get_time()
    }

    pub fn get_indices(&self) -> (Option<usize>, Option<usize>) {
        let return_under_index = match self.under_current_index.as_ref() {
            Some(val) => Some(*val.borrow()),
            _ => None,
        };

        let return_deriv_index = match self.deriv_current_index.as_ref() {
            Some(val) => Some(*val.borrow()),
            _ => None,
        };

        return (return_under_index, return_deriv_index);
    }

    pub fn get_records<'b>(&'b self) -> Option<(&'b CoinbaseRecord, &'b KalshiRecord)> {
        let coinbase_index = match self.under_current_index.as_ref() {
            Some(val) => *val.borrow(),
            _ => return None,
        };
        let return_coinbase = &self.under_data[coinbase_index];

        let deriv_index = match self.deriv_current_index.as_ref() {
            Some(val) => *val.borrow(),
            _ => return None,
        };
        let return_kalshi = &self.deriv_data.records[deriv_index];

        return Some((return_coinbase, return_kalshi));
    }
}

#[cfg(test)]
mod tests {
    use crate::read_data::data_types::KalshiRecord;

    use super::*;

    fn create_mock_cb_records(ts_start: u64, ts_end: u64) -> Vec<CoinbaseRecord> {
        let mut mock = Vec::new();
        for ts in ts_start..ts_end {
            mock.push(CoinbaseRecord::new(ts, 0.0, 0.0, 0.0));
        }
        mock
    }

    fn create_mock_kalshi_ds(ts_start: u64, ts_end: u64) -> KalshiDataset {
        let mut mock_recs = Vec::new();
        for ts in ts_start..ts_end {
            mock_recs.push(KalshiRecord::new(ts, 0.0, 0, 0));
        }
        KalshiDataset::new(mock_recs, 100.0, ts_start, ts_end)
    }

    #[test]
    fn test_binary_underlying_records() {
        let mock_cb_records = create_mock_cb_records(10, 20);
        let mock_kalshi_ds = create_mock_kalshi_ds(10, 20);

        let _stream = PairMarketStream::new(&mock_cb_records[..], mock_kalshi_ds, 1);
    }
}
