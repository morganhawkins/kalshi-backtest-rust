use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json;

use backtest::read_data::{read_kalshi::read_kalshi, read_underlying::read_coinbase};
use backtest::strats::data_types::TestResult;
use backtest::strats::delta_hedge::DeltaHedge;
use backtest::stream_data::market_stream::PairMarketStream;

fn main() {
    let max_under_pos = 0.00055;
    let min_tte_hedge = 0.0;
    let init_cash = 1.0;
    let test_func = generate_dh_pair_test(max_under_pos, min_tte_hedge, init_cash);
    let results = test_pair_streams(test_func);
    let serialized_data = serde_json::to_string(&results).expect("Failed to serialize data");

    let mut file =
        fs::File::create("/Users/morganhawkins/Projects/current/backtest/results/delta_hedge_underlying/delta_hedged_buy.json")
            .expect("failed to create file to write to");

    file.write_all(serialized_data.as_bytes())
        .expect("failed to write bytes");
}

fn generate_dh_pair_test(
    max_under_pos: f64,
    min_tte_hedge: f64,
    init_cash: f64,
) -> impl Fn(PairMarketStream) -> TestResult {
    let a = move |stream: PairMarketStream| {
        let strat = DeltaHedge::new(stream, max_under_pos, min_tte_hedge, init_cash);
        let term_port_val = strat.test();
        return term_port_val;
    };
    return a;
}

fn test_pair_streams(test_fn: impl Fn(PairMarketStream) -> TestResult) -> Vec<TestResult> {
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
        }
    }

    // reading underlying data
    let underlying_data_path =
        "/Users/morganhawkins/Projects/current/backtest/data/underlying/btc.json";
    let cb_records = read_coinbase(underlying_data_path).unwrap();
    let mut test_results = Vec::new();
    for contract_data in deriv_data {
        let relevant_data = match contract_data.relevant_slice(&cb_records) {
            Some(val) => val,
            _ => {
                println!("skipping over contract");
                continue;
            }
        };

        let stream = PairMarketStream::new(relevant_data, contract_data, 60);
        let result = test_fn(stream);
        test_results.push(result);
    }

    return test_results;
}
