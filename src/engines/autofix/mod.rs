pub mod autofix_engine;
pub mod drift_safety;
pub mod patch_generator;
pub mod patch_simulation;
pub mod snippet_generator;

pub use autofix_engine::{AutofixEngine, AutofixMode, AutofixResult};
pub use patch_generator::{PatchFile, PatchGenerator, PatchMetadata, PatchResult};
pub use snippet_generator::{BeforeAfter, FixSnippet, SnippetFormat, SnippetGenerator};
