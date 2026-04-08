use crate::codec::{bytes_to_ascii_string, bytes_to_dec_string, bytes_to_hex_display};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    Read,
    Write,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub record_type: RecordType,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
}

/// Formatted record ready for frontend display.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormattedRecord {
    pub record_type: RecordType,
    /// "0xAA, 0xBB" format
    pub hex: String,
    /// UTF-8 lossy string
    pub ascii: String,
    /// "255 0 128" format
    pub dec: String,
    /// "2024-01-01 12:00:00.123"
    pub timestamp: String,
}

pub struct RecordManager {
    records: Vec<Record>,
    /// Total received bytes (sum of data lengths for Read records).
    rx_count: u64,
    /// Total transmitted bytes (sum of data lengths for Write records).
    tx_count: u64,
}

impl RecordManager {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            rx_count: 0,
            tx_count: 0,
        }
    }

    /// Add a record and update the appropriate byte counter.
    pub fn add_record(&mut self, record: Record) {
        match record.record_type {
            RecordType::Read => self.rx_count += record.data.len() as u64,
            RecordType::Write => self.tx_count += record.data.len() as u64,
            RecordType::System => {}
        }
        self.records.push(record);
    }

    /// Convert all records to `FormattedRecord` for frontend consumption.
    pub fn get_records(&self) -> Vec<FormattedRecord> {
        self.records
            .iter()
            .map(|r| FormattedRecord {
                record_type: r.record_type.clone(),
                hex: bytes_to_hex_display(&r.data),
                ascii: bytes_to_ascii_string(&r.data),
                dec: bytes_to_dec_string(&r.data),
                timestamp: r.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            })
            .collect()
    }

    /// Clear all records and reset both byte counters.
    pub fn clear_records(&mut self) {
        self.records.clear();
        self.rx_count = 0;
        self.tx_count = 0;
    }

    /// Clear only records of the given type; byte counters are NOT reset.
    pub fn clear_records_by_type(&mut self, record_type: &RecordType) {
        self.records.retain(|r| {
            !matches!(
                (&r.record_type, record_type),
                (RecordType::Read, RecordType::Read)
                    | (RecordType::Write, RecordType::Write)
                    | (RecordType::System, RecordType::System)
            )
        });
    }

    /// Serialise all formatted records to a pretty-printed JSON string.
    pub fn export_records(&self) -> String {
        serde_json::to_string_pretty(&self.get_records()).unwrap_or_else(|_| "[]".to_string())
    }

    /// Total bytes received (Read records).
    pub fn rx_count(&self) -> u64 {
        self.rx_count
    }

    /// Total bytes transmitted (Write records).
    pub fn tx_count(&self) -> u64 {
        self.tx_count
    }
}

impl Default for RecordManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_record(record_type: RecordType, data: Vec<u8>) -> Record {
        Record {
            record_type,
            data,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_new_manager_empty() {
        let mgr = RecordManager::new();
        assert_eq!(mgr.rx_count(), 0);
        assert_eq!(mgr.tx_count(), 0);
        assert!(mgr.get_records().is_empty());
    }

    #[test]
    fn test_add_read_record_increments_rx_count() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Read, vec![0xAA, 0xBB, 0xCC]));
        assert_eq!(mgr.rx_count(), 3);
        assert_eq!(mgr.tx_count(), 0);
    }

    #[test]
    fn test_add_write_record_increments_tx_count() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Write, vec![0x01, 0x02]));
        assert_eq!(mgr.tx_count(), 2);
        assert_eq!(mgr.rx_count(), 0);
    }

    #[test]
    fn test_system_record_does_not_affect_counts() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::System, vec![0x00]));
        assert_eq!(mgr.rx_count(), 0);
        assert_eq!(mgr.tx_count(), 0);
    }

    #[test]
    fn test_get_records_formatting() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Read, vec![0xAA, 0xBB]));
        let formatted = mgr.get_records();
        assert_eq!(formatted.len(), 1);
        assert_eq!(formatted[0].hex, "0xAA, 0xBB");
        assert_eq!(formatted[0].dec, "170 187");
        // ascii field is lossy UTF-8; just verify it is non-empty
        assert!(!formatted[0].timestamp.is_empty());
    }

    #[test]
    fn test_clear_records_resets_counts() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Read, vec![1, 2, 3]));
        mgr.add_record(make_record(RecordType::Write, vec![4, 5]));
        mgr.clear_records();
        assert_eq!(mgr.rx_count(), 0);
        assert_eq!(mgr.tx_count(), 0);
        assert!(mgr.get_records().is_empty());
    }

    #[test]
    fn test_clear_records_by_type_keeps_other_types() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Read, vec![1]));
        mgr.add_record(make_record(RecordType::Write, vec![2]));
        mgr.add_record(make_record(RecordType::System, vec![3]));

        mgr.clear_records_by_type(&RecordType::Read);

        let records = mgr.get_records();
        assert_eq!(records.len(), 2);
        // Counts are NOT reset
        assert_eq!(mgr.rx_count(), 1);
        assert_eq!(mgr.tx_count(), 1);
    }

    #[test]
    fn test_clear_records_by_type_write() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Write, vec![0xFF, 0x00]));
        mgr.add_record(make_record(RecordType::Read, vec![0x01]));
        mgr.clear_records_by_type(&RecordType::Write);

        let records = mgr.get_records();
        assert_eq!(records.len(), 1);
        assert!(matches!(records[0].record_type, RecordType::Read));
    }

    #[test]
    fn test_export_records_is_valid_json() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Read, vec![0xDE, 0xAD]));
        let json = mgr.export_records();
        // Should parse without error
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_timestamp_format() {
        let mut mgr = RecordManager::new();
        mgr.add_record(make_record(RecordType::Read, vec![0x00]));
        let records = mgr.get_records();
        // Rough check: "YYYY-MM-DD HH:MM:SS.mmm" => 23 chars
        assert_eq!(records[0].timestamp.len(), 23);
    }
}
