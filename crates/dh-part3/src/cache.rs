use dh_core::Result;
use dh_ir::TableBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageCacheKey {
    pub content_sha256: String,
    pub page: u32,
    pub extractor_version: String,
    pub policy_versions: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct PageCache {
    entries: Vec<(PageCacheKey, Vec<TableBlock>)>,
}

impl PageCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &PageCacheKey) -> Option<&Vec<TableBlock>> {
        self.entries
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, blocks)| blocks)
    }

    pub fn insert(&mut self, key: PageCacheKey, blocks: Vec<TableBlock>) {
        self.entries.push((key, blocks));
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn invalidate_for_content(&mut self, content_sha256: &str) -> Result<()> {
        todo!("PageCache::invalidate_for_content")
    }
}
