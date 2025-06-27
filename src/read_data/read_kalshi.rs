use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use serde_json::Value;

use super::data_types::{KalshiRecord, KalshiDataset};

pub fn read_kalshi<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error + Send + Sync>>{

    // if !fp.ends_with(".json"){
    //     return Err("file format must be json".into());
    // }

    let file = File::open(path)?;

    // Create a buffered reader to efficiently read the file's contents
    let reader = BufReader::new(file);

    // Parse the JSON data from the reader
    let record: Value = serde_json::from_reader(reader)?;

    match record {
        Value::Array(arr) => {
            for record in arr{
                let kalshi_record = serde_json::from_value::<KalshiRecord>(record);
                
            }
        }, 
        _ => return Err("not an Array".into())
    }


    Ok(())
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_mock() {


    }




}