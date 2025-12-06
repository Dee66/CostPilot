pub mod autofix_engine;
pub mod snippet_generator;
pub mod patch_generator;
pub mod patch_simulation;
pub mod drift_safety;

pub use autofix_engine::{AutofixEngine, AutofixMode, AutofixResult};
pub use snippet_generator::{FixSnippet, SnippetGenerator, SnippetFormat, BeforeAfter};
pub use patch_generator::{PatchFile, PatchGenerator, PatchResult, PatchMetadata};
