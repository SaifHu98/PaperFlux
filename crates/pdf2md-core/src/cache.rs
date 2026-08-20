use std::collections::HashMap;
use std::sync::RwLock;
use pdf2md_ast::Section;
use pdf2md_pdf::elements::RawPage;

pub struct PageCache {
    cache: RwLock<HashMap<u64, Section>>,
    max_entries: usize,
}

impl PageCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(max_entries)),
            max_entries,
        }
    }

    pub fn compute_page_hash(page: &RawPage) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        page.page_number.hash(&mut hasher);
        page.width.to_bits().hash(&mut hasher);
        page.height.to_bits().hash(&mut hasher);
        page.text_spans.len().hash(&mut hasher);

        for span in &page.text_spans {
            span.text.hash(&mut hasher);
            span.bbox.x.to_bits().hash(&mut hasher);
            span.bbox.y.to_bits().hash(&mut hasher);
        }

        hasher.finish()
    }

    pub fn get(&self, hash: u64) -> Option<Section> {
        let read_guard = self.cache.read().ok()?;
        read_guard.get(&hash).cloned()
    }

    pub fn insert(&self, hash: u64, section: Section) {
        if let Ok(mut write_guard) = self.cache.write() {
            if write_guard.len() < self.max_entries {
                write_guard.insert(hash, section);
            }
        }
    }

    pub fn clear(&self) {
        if let Ok(mut write_guard) = self.cache.write() {
            write_guard.clear();
        }
    }
}

impl Default for PageCache {
    fn default() -> Self {
        Self::new(256)
    }
}
