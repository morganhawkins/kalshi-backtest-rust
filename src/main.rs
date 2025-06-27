use std::{path::{Path, PathBuf}, fs};

use backtest::read_data::{read_kalshi::read_kalshi, read_underlying::read_coinbase};

// TODO: collect contract data paths, underlying paths, and meta data before handing references
fn main() {
    let root = Path::new("/Users/morganhawkins/Projects/current/backtest/data/kalshi_step/btc/");

    // initializing data to be rewritten on iter
    let mut data_files: fs::ReadDir;
    let mut data_path: PathBuf;
    

    // reading all kalshi data files
    let dates = fs::read_dir(root).unwrap();
    for date in dates {
        let date_dir_entry = match date {
            Ok(val) => val,
            _ => continue
        };

        data_files = match fs::read_dir(date_dir_entry.path()){
            Ok(val) => val,
            _ => continue
        };
    
        for data_file in data_files {
            data_path = data_file.unwrap().path();
            read_kalshi(data_path).unwrap();
            
        }
    }

    // reading udnerlying data
    let underlying_data_path = "/Users/morganhawkins/Projects/current/backtest/data/underlying/btc.json";
    let cb_records = read_coinbase(underlying_data_path).unwrap();
    

}

