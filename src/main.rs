use std::{
    fs,
    path::{Path, PathBuf},
};

use backtest::read_data::{
    data_types::CoinbaseRecord, read_kalshi::read_kalshi, read_underlying::read_coinbase,
};
use backtest::stream_data::market_stream::PairMarketStream;
use backtest::stream_data::timers::{DeltaTimer, Timer};
use statrs::function::erf::{erf, erf_inv};

// TODO: collect contract data paths, underlying paths, and meta data before handing references
fn main() {
    println!("{}", erf_inv(1.0));

    return;
    let root = Path::new("/Users/morganhawkins/Projects/current/backtest/data/kalshi_step/btc/");

    // initializing data to be rewritten on iter
    let mut data_files: fs::ReadDir;
    let mut data_path: PathBuf;

    let mut deriv_data = Vec::new();

    // reading all kalshi data files
    let dates = fs::read_dir(root).unwrap();
    for date in dates {
        let date_dir_entry = match date {
            Ok(val) => val,
            _ => continue,
        };

        data_files = match fs::read_dir(date_dir_entry.path()) {
            Ok(val) => val,
            _ => continue,
        };

        for data_file in data_files {
            data_path = data_file.unwrap().path();
            deriv_data.push(read_kalshi(data_path).unwrap());
            // break
        }
    }

    // reading underlying data
    let underlying_data_path =
        "/Users/morganhawkins/Projects/current/backtest/data/underlying/btc.json";
    let cb_records = read_coinbase(underlying_data_path).unwrap();

    for contract_data in deriv_data {
        let relevant_data = match contract_data.relevant_slice(&cb_records) {
            Some(val) => val,
            _ => {
                println!("skipping over contract");
                continue;
            }
        };

        let stream = PairMarketStream::new(relevant_data, contract_data, 60);

        loop {
            if stream.get_time().is_none() {
                break;
            } else {
                stream.cycle();
            }
        }
    }
}
