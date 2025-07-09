use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub metadata: Metadata,
    pub manifest: HashMap<String, ManifestItem>,
    pub spine: Vec<SpineItem>,
    pub toc: Option<TableOfContents>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub subjects: Vec<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub content: Option<Vec<u8>>,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SpineItem {
    pub id: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    pub title: Option<String>,
    pub items: Vec<TocItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    pub title: String,
    pub href: Option<String>,
    pub children: Vec<TocItem>,
}

impl Book {
    pub fn new() -> Self {
        Self {
            metadata: Metadata::new(),
            manifest: HashMap::new(),
            spine: Vec::new(),
            toc: None,
        }
    }
    
    pub fn add_manifest_item(&mut self, id: String, item: ManifestItem) {
        self.manifest.insert(id, item);
    }
    
    pub fn add_spine_item(&mut self, item: SpineItem) {
        self.spine.push(item);
    }
    
    pub fn get_manifest_item(&self, id: &str) -> Option<&ManifestItem> {
        self.manifest.get(id)
    }
    
    pub fn get_manifest_item_mut(&mut self, id: &str) -> Option<&mut ManifestItem> {
        self.manifest.get_mut(id)
    }
    
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
    
    pub fn set_toc(&mut self, toc: TableOfContents) {
        self.toc = Some(toc);
    }
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            title: None,
            author: None,
            language: None,
            identifier: None,
            publisher: None,
            description: None,
            subjects: Vec::new(),
            date: None,
        }
    }
    
    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }
    
    pub fn set_author(&mut self, author: String) {
        self.author = Some(author);
    }
    
    pub fn set_language(&mut self, language: String) {
        self.language = Some(language);
    }
    
    pub fn set_identifier(&mut self, identifier: String) {
        self.identifier = Some(identifier);
    }
    
    pub fn set_publisher(&mut self, publisher: String) {
        self.publisher = Some(publisher);
    }
    
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }
    
    pub fn add_subject(&mut self, subject: String) {
        self.subjects.push(subject);
    }
    
    pub fn set_date(&mut self, date: String) {
        self.date = Some(date);
    }
}

impl ManifestItem {
    pub fn new(id: String, href: String, media_type: String) -> Self {
        Self {
            id,
            href,
            media_type,
            content: None,
            properties: Vec::new(),
        }
    }
    
    pub fn with_content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content);
        self
    }
    
    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }
}

impl SpineItem {
    pub fn new(id: String, href: String) -> Self {
        Self { id, href }
    }
}

impl TableOfContents {
    pub fn new() -> Self {
        Self {
            title: None,
            items: Vec::new(),
        }
    }
    
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    
    pub fn add_item(&mut self, item: TocItem) {
        self.items.push(item);
    }
}

impl TocItem {
    pub fn new(title: String) -> Self {
        Self {
            title,
            href: None,
            children: Vec::new(),
        }
    }
    
    pub fn with_href(mut self, href: String) -> Self {
        self.href = Some(href);
        self
    }
    
    pub fn add_child(&mut self, child: TocItem) {
        self.children.push(child);
    }
} 