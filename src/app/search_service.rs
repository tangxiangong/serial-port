use crate::core::{
    records::RecordEntry,
    search::{SearchIndex, SearchResult},
};

#[derive(Debug, Default)]
pub struct SearchService {
    index: SearchIndex,
}

impl SearchService {
    pub async fn rebuild(&mut self, records: &[RecordEntry]) {
        self.index.rebuild(records);
    }

    pub async fn append(&mut self, start_index: usize, records: &[RecordEntry]) {
        self.index.append(start_index, records);
    }

    pub async fn query(&self, query: &str) -> Vec<SearchResult> {
        self.index.query(query)
    }
}
