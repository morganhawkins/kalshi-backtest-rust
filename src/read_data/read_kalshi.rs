use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use serde_json;
use regex::Regex;

use super::data_types::{KalshiRecord, KalshiDataset};

fn chunk_kalshi_records<'a>(text: &'a String, re: &Regex) -> impl Iterator<Item = &'a str> {
    let chunks = re.find_iter(text).map(|m| m.as_str());
    chunks
}

pub fn read_kalshi<P: AsRef<Path>>(path: P) -> Result<Vec<KalshiRecord>, Box<dyn Error>>{
    // compile regex to chunk array into objects
    let pattern= r"\{.*?\}";
    let re = Regex::new(pattern).unwrap(); 

    // read to string and chunk
    let contents = read_to_string(path)?;
    let chunks = chunk_kalshi_records(&contents, &re);
    
    // TODO: remove unwrap to handle deserialization error
    let records: Vec<KalshiRecord> = chunks.map(|s| serde_json::from_str::<KalshiRecord>(s).unwrap()).collect();
    
    // getting meta data for dataset
    let expir_ts = (records[0].tte*(3600_f64)) as u64 + records[0].ts;
    

    Ok(records)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_mock() {

    }

}