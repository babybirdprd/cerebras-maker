// Knowledge Base Module
// Handles pre-existing research, documentation, and web research for LLM consumption

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Type of knowledge document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocumentType {
    PRD,
    APIReference,
    DesignSpec,
    UserStory,
    TechnicalSpec,
    Architecture,
    StyleGuide,
    WebResearch,
    MeetingNotes,
    Other,
}

impl DocumentType {
    /// Parse document type from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "prd" => DocumentType::PRD,
            "apireference" | "api_reference" | "api" => DocumentType::APIReference,
            "designspec" | "design_spec" | "design" => DocumentType::DesignSpec,
            "userstory" | "user_story" | "story" => DocumentType::UserStory,
            "technicalspec" | "technical_spec" | "tech_spec" => DocumentType::TechnicalSpec,
            "architecture" => DocumentType::Architecture,
            "style_guide" | "styleguide" | "style" => DocumentType::StyleGuide,
            "web_research" | "webresearch" | "web" => DocumentType::WebResearch,
            "meetingnotes" | "meeting_notes" | "notes" => DocumentType::MeetingNotes,
            _ => DocumentType::Other,
        }
    }

    /// Get priority for context ordering (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            DocumentType::PRD => 0,           // Highest priority - core requirements
            DocumentType::Architecture => 1,   // High - system design
            DocumentType::TechnicalSpec => 2,  // High - technical details
            DocumentType::DesignSpec => 3,     // Medium-high - UI/UX specs
            DocumentType::APIReference => 4,   // Medium - API docs
            DocumentType::UserStory => 5,      // Medium - user context
            DocumentType::StyleGuide => 6,     // Lower - style guidelines
            DocumentType::WebResearch => 7,    // Lower - supplementary research
            DocumentType::MeetingNotes => 8,   // Lowest - informal notes
            DocumentType::Other => 9,          // Catch-all
        }
    }

    /// Get display name for the document type
    pub fn display_name(&self) -> &'static str {
        match self {
            DocumentType::PRD => "Product Requirements",
            DocumentType::APIReference => "API Reference",
            DocumentType::DesignSpec => "Design Specification",
            DocumentType::UserStory => "User Story",
            DocumentType::TechnicalSpec => "Technical Specification",
            DocumentType::Architecture => "Architecture",
            DocumentType::StyleGuide => "Style Guide",
            DocumentType::WebResearch => "Web Research",
            DocumentType::MeetingNotes => "Meeting Notes",
            DocumentType::Other => "Other",
        }
    }
}

/// Document classifier for auto-detection
pub struct DocumentClassifier;

impl DocumentClassifier {
    /// Classify document type based on content analysis
    pub fn classify(content: &str, filename: &str) -> DocumentType {
        let content_lower = content.to_lowercase();
        let filename_lower = filename.to_lowercase();

        // Check filename first for strong hints
        if filename_lower.contains("prd") || filename_lower.contains("requirements") {
            return DocumentType::PRD;
        }
        if filename_lower.contains("api") || filename_lower.contains("swagger") || filename_lower.contains("openapi") {
            return DocumentType::APIReference;
        }
        if filename_lower.contains("architecture") || filename_lower.contains("arch") {
            return DocumentType::Architecture;
        }
        if filename_lower.contains("design") || filename_lower.contains("figma") || filename_lower.contains("mockup") {
            return DocumentType::DesignSpec;
        }
        if filename_lower.contains("style") || filename_lower.contains("guideline") {
            return DocumentType::StyleGuide;
        }
        if filename_lower.contains("meeting") || filename_lower.contains("notes") || filename_lower.contains("minutes") {
            return DocumentType::MeetingNotes;
        }

        // Content-based classification using keyword scoring
        let scores = Self::score_content(&content_lower);

        // Find the type with highest score (min score of 2 to avoid false positives)
        scores.into_iter()
            .filter(|(_, score)| *score >= 2)
            .max_by_key(|(_, score)| *score)
            .map(|(doc_type, _)| doc_type)
            .unwrap_or(DocumentType::Other)
    }

    /// Score content against document type keywords
    fn score_content(content: &str) -> Vec<(DocumentType, usize)> {
        let prd_keywords = ["requirement", "feature", "user story", "acceptance criteria",
            "scope", "deliverable", "milestone", "stakeholder", "mvp", "must have", "should have"];
        let api_keywords = ["endpoint", "request", "response", "http", "get", "post", "put",
            "delete", "json", "rest", "graphql", "authentication", "authorization", "bearer"];
        let arch_keywords = ["component", "service", "database", "infrastructure", "deployment",
            "scalability", "microservice", "architecture", "diagram", "system design", "layer"];
        let design_keywords = ["wireframe", "mockup", "ui", "ux", "layout", "color", "typography",
            "component", "screen", "interaction", "prototype", "figma"];
        let tech_spec_keywords = ["implementation", "algorithm", "data structure", "performance",
            "optimization", "constraint", "specification", "technical", "schema"];
        let style_keywords = ["naming convention", "code style", "formatting", "linting",
            "best practice", "pattern", "convention", "guideline"];
        let meeting_keywords = ["attendees", "agenda", "action item", "discussed", "decided",
            "follow-up", "meeting", "date:", "participants"];

        vec![
            (DocumentType::PRD, Self::count_keywords(content, &prd_keywords)),
            (DocumentType::APIReference, Self::count_keywords(content, &api_keywords)),
            (DocumentType::Architecture, Self::count_keywords(content, &arch_keywords)),
            (DocumentType::DesignSpec, Self::count_keywords(content, &design_keywords)),
            (DocumentType::TechnicalSpec, Self::count_keywords(content, &tech_spec_keywords)),
            (DocumentType::StyleGuide, Self::count_keywords(content, &style_keywords)),
            (DocumentType::MeetingNotes, Self::count_keywords(content, &meeting_keywords)),
        ]
    }

    /// Count keyword occurrences in content
    fn count_keywords(content: &str, keywords: &[&str]) -> usize {
        keywords.iter()
            .filter(|kw| content.contains(*kw))
            .count()
    }
}

/// A knowledge document (PRD, API docs, design specs, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub name: String,
    pub content: String,
    pub doc_type: DocumentType,
    pub metadata: HashMap<String, String>,
    pub auto_classified: bool,
    pub word_count: usize,
}

impl KnowledgeDocument {
    pub fn new(name: String, content: String, doc_type: DocumentType) -> Self {
        let word_count = content.split_whitespace().count();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content,
            doc_type,
            metadata: HashMap::new(),
            auto_classified: false,
            word_count,
        }
    }

    /// Create a document with auto-classification
    pub fn new_auto_classified(name: String, content: String) -> Self {
        let doc_type = DocumentClassifier::classify(&content, &name);
        let word_count = content.split_whitespace().count();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content,
            doc_type,
            metadata: HashMap::new(),
            auto_classified: true,
            word_count,
        }
    }

    /// Get estimated token count (rough approximation: ~0.75 tokens per word)
    pub fn estimated_tokens(&self) -> usize {
        (self.word_count as f32 * 0.75) as usize
    }
}

/// Web research item (crawled content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResearchItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub content: String,
    pub crawled_at: DateTime<Utc>,
}

impl WebResearchItem {
    pub fn new(url: String, title: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            title,
            content,
            crawled_at: Utc::now(),
        }
    }
}

/// Knowledge base containing documents and web research
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBase {
    pub documents: Vec<KnowledgeDocument>,
    pub web_research: Vec<WebResearchItem>,
    pub project_path: Option<PathBuf>,
}

impl KnowledgeBase {
    /// Create empty knowledge base
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            web_research: Vec::new(),
            project_path: None,
        }
    }

    /// Add a document to the knowledge base with explicit type
    pub fn add_document(&mut self, name: String, content: String, doc_type: DocumentType) -> String {
        let doc = KnowledgeDocument::new(name, content, doc_type);
        let id = doc.id.clone();
        self.documents.push(doc);
        id
    }

    /// Add a document with auto-classification
    pub fn add_document_auto(&mut self, name: String, content: String) -> (String, DocumentType) {
        let doc = KnowledgeDocument::new_auto_classified(name, content);
        let id = doc.id.clone();
        let doc_type = doc.doc_type.clone();
        self.documents.push(doc);
        (id, doc_type)
    }

    /// Add crawled web content to the knowledge base
    pub fn add_web_research(&mut self, url: String, title: String, content: String) -> String {
        let item = WebResearchItem::new(url, title, content);
        let id = item.id.clone();
        self.web_research.push(item);
        id
    }

    /// Remove a document by ID
    pub fn remove_document(&mut self, id: &str) -> bool {
        let original_len = self.documents.len();
        self.documents.retain(|doc| doc.id != id);
        self.documents.len() != original_len
    }

    /// Get all documents
    pub fn get_all_documents(&self) -> &Vec<KnowledgeDocument> {
        &self.documents
    }

    /// Get total estimated token count
    pub fn total_tokens(&self) -> usize {
        let doc_tokens: usize = self.documents.iter().map(|d| d.estimated_tokens()).sum();
        let web_tokens: usize = self.web_research.iter()
            .map(|w| (w.content.split_whitespace().count() as f32 * 0.75) as usize)
            .sum();
        doc_tokens + web_tokens
    }

    /// Get documents sorted by priority
    pub fn documents_by_priority(&self) -> Vec<&KnowledgeDocument> {
        let mut docs: Vec<_> = self.documents.iter().collect();
        docs.sort_by_key(|d| d.doc_type.priority());
        docs
    }

    /// Compile all knowledge into a single context string for LLM consumption
    /// Uses priority ordering and respects token budget
    pub fn compile_context(&self) -> String {
        self.compile_context_with_budget(None)
    }

    /// Compile context with optional token budget
    pub fn compile_context_with_budget(&self, max_tokens: Option<usize>) -> String {
        let mut context = String::new();
        let mut current_tokens = 0usize;
        let budget = max_tokens.unwrap_or(usize::MAX);

        // Get documents sorted by priority
        let sorted_docs = self.documents_by_priority();

        // Group documents by type for better organization
        let mut grouped: HashMap<&DocumentType, Vec<&KnowledgeDocument>> = HashMap::new();
        for doc in &sorted_docs {
            grouped.entry(&doc.doc_type).or_default().push(doc);
        }

        // Build context header
        if !self.documents.is_empty() || !self.web_research.is_empty() {
            let header = format!(
                "# Knowledge Base Context\n\n*{} documents, {} web research items, ~{} estimated tokens*\n\n",
                self.documents.len(),
                self.web_research.len(),
                self.total_tokens()
            );
            context.push_str(&header);
            current_tokens += 50; // Rough estimate for header
        }

        // Add documents by priority groups
        let type_order = [
            DocumentType::PRD,
            DocumentType::Architecture,
            DocumentType::TechnicalSpec,
            DocumentType::DesignSpec,
            DocumentType::APIReference,
            DocumentType::UserStory,
            DocumentType::StyleGuide,
            DocumentType::MeetingNotes,
            DocumentType::Other,
        ];

        for doc_type in &type_order {
            if let Some(docs) = grouped.get(doc_type) {
                if docs.is_empty() {
                    continue;
                }

                let section_header = format!("## {}\n\n", doc_type.display_name());
                let section_tokens = 10; // Estimate for header

                if current_tokens + section_tokens > budget {
                    context.push_str("\n\n*[Context truncated due to token budget]*\n");
                    break;
                }

                context.push_str(&section_header);
                current_tokens += section_tokens;

                for doc in docs {
                    let doc_tokens = doc.estimated_tokens();

                    if current_tokens + doc_tokens > budget {
                        // Try to include a summary instead
                        let summary = Self::summarize_document(doc, budget - current_tokens);
                        context.push_str(&format!("### {} (summarized)\n{}\n\n", doc.name, summary));
                        context.push_str("*[Remaining documents truncated]*\n\n");
                        break;
                    }

                    context.push_str(&format!("### {}\n", doc.name));
                    context.push_str(&doc.content);
                    context.push_str("\n\n---\n\n");
                    current_tokens += doc_tokens;
                }
            }
        }

        // Add web research section (lower priority)
        if !self.web_research.is_empty() && current_tokens < budget {
            context.push_str("## Web Research\n\n");
            current_tokens += 10;

            for item in &self.web_research {
                let item_tokens = (item.content.split_whitespace().count() as f32 * 0.75) as usize;

                if current_tokens + item_tokens > budget {
                    context.push_str("*[Web research truncated due to token budget]*\n");
                    break;
                }

                context.push_str(&format!("### {} \n*Source: {}*\n\n", item.title, item.url));
                context.push_str(&item.content);
                context.push_str("\n\n---\n\n");
                current_tokens += item_tokens;
            }
        }

        context
    }

    /// Create a brief summary of a document (first N tokens worth)
    fn summarize_document(doc: &KnowledgeDocument, max_tokens: usize) -> String {
        let words: Vec<&str> = doc.content.split_whitespace().collect();
        let word_limit = ((max_tokens as f32) / 0.75) as usize;
        let truncated: String = words.iter().take(word_limit).cloned().collect::<Vec<_>>().join(" ");
        format!("{}...", truncated)
    }

    /// Compile context for L1 Interrogator (optimized for requirements analysis)
    pub fn compile_for_interrogator(&self) -> String {
        // For interrogation, prioritize PRDs, user stories, and design specs
        let mut context = String::from("# Pre-existing Knowledge Context\n\n");
        context.push_str("*The user has provided the following background materials:*\n\n");

        // First: PRDs and requirements
        let prds: Vec<_> = self.documents.iter()
            .filter(|d| matches!(d.doc_type, DocumentType::PRD | DocumentType::UserStory))
            .collect();

        if !prds.is_empty() {
            context.push_str("## Requirements & User Stories\n\n");
            for doc in prds {
                context.push_str(&format!("### {}\n{}\n\n", doc.name, doc.content));
            }
        }

        // Second: Technical context
        let tech_docs: Vec<_> = self.documents.iter()
            .filter(|d| matches!(d.doc_type, DocumentType::Architecture | DocumentType::TechnicalSpec | DocumentType::APIReference))
            .collect();

        if !tech_docs.is_empty() {
            context.push_str("## Technical Context\n\n");
            for doc in tech_docs {
                context.push_str(&format!("### {} ({})\n{}\n\n", doc.name, doc.doc_type.display_name(), doc.content));
            }
        }

        // Third: Design context
        let design_docs: Vec<_> = self.documents.iter()
            .filter(|d| matches!(d.doc_type, DocumentType::DesignSpec | DocumentType::StyleGuide))
            .collect();

        if !design_docs.is_empty() {
            context.push_str("## Design Context\n\n");
            for doc in design_docs {
                context.push_str(&format!("### {}\n{}\n\n", doc.name, doc.content));
            }
        }

        // Fourth: Web research
        if !self.web_research.is_empty() {
            context.push_str("## Supplementary Research\n\n");
            for item in &self.web_research {
                context.push_str(&format!("### {} ({})\n{}\n\n", item.title, item.url, item.content));
            }
        }

        context
    }
}

