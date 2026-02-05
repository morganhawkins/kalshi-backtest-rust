use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::data_models::kalshi::KalshiOrderbookUpdate;

/// Reads a vector of KalshiOrderbookUpdates from a binary file.
pub fn read_orderbook_updates<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<KalshiOrderbookUpdate>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let updates: Vec<KalshiOrderbookUpdate> = bincode::deserialize_from(reader)?;
    Ok(updates)
}

/// Reads orderbook updates from raw bytes.
pub fn read_orderbook_updates_from_bytes(
    bytes: &[u8],
) -> Result<Vec<KalshiOrderbookUpdate>, Box<dyn std::error::Error>> {
    let updates: Vec<KalshiOrderbookUpdate> = bincode::deserialize(bytes)?;
    Ok(updates)
}

/// Writes a vector of KalshiOrderbookUpdates to a binary file.
pub fn write_orderbook_updates<P: AsRef<Path>>(
    path: P,
    updates: &[KalshiOrderbookUpdate],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    bincode::serialize_into(file, updates)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_roundtrip() {
        let updates = vec![
            KalshiOrderbookUpdate::new(1000, 50, 10),
            KalshiOrderbookUpdate::new(1001, 50, -5),
        ];

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        write_orderbook_updates(&path, &updates).unwrap();
        let read_back = read_orderbook_updates(&path).unwrap();

        assert_eq!(read_back.len(), 2);
        assert_eq!(read_back[0].ts, 1000);
        assert_eq!(read_back[0].price, 50);
        assert_eq!(read_back[0].delta, 10);
        assert_eq!(read_back[1].ts, 1001);
        assert_eq!(read_back[1].price, 51);
        assert_eq!(read_back[1].delta, -5);
    }
}
