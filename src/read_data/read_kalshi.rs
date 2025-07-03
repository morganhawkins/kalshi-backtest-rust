use regex::Regex;
use serde_json;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

use super::data_types::{KalshiDataset, KalshiRecord};

fn chunk_kalshi_records<'a>(text: &'a String, re: &Regex) -> impl Iterator<Item = &'a str> {
    let chunks = re.find_iter(text).map(|m| m.as_str());
    chunks
}

pub fn read_kalshi<P: AsRef<Path>>(path: P) -> Result<KalshiDataset, Box<dyn Error>> {
    // compile regex to chunk array into objects
    let pattern = r"\{.*?\}";
    let re = Regex::new(pattern).unwrap();

    // read to string and chunk
    let contents = read_to_string(&path)?;
    let chunks = chunk_kalshi_records(&contents, &re);

    // TODO: remove unwrap to handle deserialization error
    let records: Vec<KalshiRecord> = chunks
        .map(|s| serde_json::from_str::<KalshiRecord>(s).unwrap())
        .collect();

    // getting meta data for dataset
    let expir_ts = (records[0].tte * (3600_f64)) as u64 + records[0].ts;
    let start_ts = records[0].ts;
    let strike_fn = match path.as_ref().file_name() {
        Some(val) => val,
        None => {
            return Err(format!("unable to turn `path` into string: {:?}", path.as_ref()).into());
        }
    };
    let strike_split_str: Vec<&str> = strike_fn.to_str().unwrap().split(".").into_iter().collect();
    let strike = match str::parse::<f64>(strike_split_str[0]) {
        Ok(val) => val,
        Err(_) => {
            return Err(format!(
                "unable to parse file name w/o extenion into f64 strike price: {:?}",
                strike_split_str
            )
            .into());
        }
    };

    let dataset = KalshiDataset {
        records,
        strike,
        start_ts,
        expir_ts,
    };
    Ok(dataset)
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_read_mock() {}
}
