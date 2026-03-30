use serde::{Deserialize, Serialize};

use crate::core::codecs::render_payload;
use crate::core::records::RecordEntry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    pub index: usize,
    pub kind: String,
    pub timestamp: String,
    pub content: String,
    pub length: usize,
}

#[derive(Debug, Default, Clone)]
pub struct SearchIndex {
    entries: Vec<SearchResult>,
}

impl SearchIndex {
    pub fn rebuild(&mut self, records: &[RecordEntry]) {
        self.entries = records
            .iter()
            .enumerate()
            .map(|(index, record)| SearchResult {
                index,
                kind: record.kind.label().to_string(),
                timestamp: record.time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                content: render_payload(&record.data, record.display),
                length: record.data.len(),
            })
            .collect();
    }

    pub fn append(&mut self, start_index: usize, records: &[RecordEntry]) {
        self.entries.extend(
            records
                .iter()
                .enumerate()
                .map(|(offset, record)| SearchResult {
                    index: start_index + offset,
                    kind: record.kind.label().to_string(),
                    timestamp: record.time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    content: render_payload(&record.data, record.display),
                    length: record.data.len(),
                }),
        );
    }

    pub fn query(&self, query: &str) -> Vec<SearchResult> {
        let trimmed = query.trim().to_lowercase();
        if trimmed.is_empty() {
            return self.entries.iter().take(50).cloned().collect();
        }

        self.entries
            .iter()
            .filter(|entry| {
                entry.kind.to_lowercase().contains(&trimmed)
                    || entry.timestamp.to_lowercase().contains(&trimmed)
                    || entry.content.to_lowercase().contains(&trimmed)
            })
            .take(50)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::SearchIndex;
    use crate::core::records::{DisplayMode, RecordEntry, RecordKind};

    #[test]
    fn query_matches_rendered_content() {
        let mut index = SearchIndex::default();
        let records = vec![RecordEntry {
            kind: RecordKind::Write,
            data: b"AT+GMR".to_vec(),
            time: Utc::now(),
            display: DisplayMode::Ascii,
        }];
        index.rebuild(&records);
        assert_eq!(index.query("GMR").len(), 1);
    }
}
