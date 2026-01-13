// Cerebras-MAKER: Core MAKER Framework Module
// PRD Section 4: The Logic Layer - Rhai "Code Mode" Runtime

pub mod atom;
pub mod runtime;
pub mod shadow_git;
pub mod voting;

// Re-exports for convenience
pub use atom::{AtomType, AtomResult};
pub use runtime::CodeModeRuntime;
pub use shadow_git::ShadowGit;
pub use voting::{run_consensus, ConsensusConfig, ConsensusResult};

