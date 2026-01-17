// Cerebras-MAKER: Core MAKER Framework Module
// PRD Section 4: The Logic Layer - Rhai "Code Mode" Runtime

pub mod ast_edit;
pub mod atom;
pub mod atom_bridge;
pub mod rlm;
pub mod runtime;
pub mod shadow_git;
pub mod voting;
pub mod web_research_bridge;

// Re-exports for convenience
pub use ast_edit::{AstEditor, SupportedLanguage, SyntaxValidationResult, SyntaxError};
pub use atom::{AtomType, AtomResult, SpawnFlags};
pub use rlm::{RLMConfig, RLMContextStore, RLMResult, RLMTrajectoryStep, RLMOperation, ContextType, ContextMetadata, SharedRLMContextStore, create_shared_store, RLMAction, RLMExecutionState};
pub use runtime::CodeModeRuntime;
pub use shadow_git::ShadowGit;
pub use voting::{run_consensus, ConsensusConfig, ConsensusResult};
pub use web_research_bridge::{init_web_research_worker, crawl_url_sync, research_docs_sync, extract_content_sync};

