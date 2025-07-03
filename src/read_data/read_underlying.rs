use regex::Regex;
use serde_json;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

use super::data_types::CoinbaseRecord;

fn chunk_coinbase_records<'a>(text: &'a String, re: &Regex) -> impl Iterator<Item = &'a str> {
    let chunks = re.find_iter(text).map(|m| m.as_str());
    chunks
}

pub fn read_coinbase<P: AsRef<Path>>(path: P) -> Result<Vec<CoinbaseRecord>, Box<dyn Error>> {
    // compile regex to chunk array into objects
    let pattern = r"\{.*?\}";
    let re = Regex::new(pattern).unwrap();

    // read to string and chunk
    let contents = read_to_string(path)?;
    let chunks = chunk_coinbase_records(&contents, &re);

    // TODO: remove unwrap to handle deserialization error
    let records: Vec<CoinbaseRecord> = chunks
        .map(|s| serde_json::from_str::<CoinbaseRecord>(s).unwrap())
        .collect();

    Ok(records)
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_read_mock() {}
}
