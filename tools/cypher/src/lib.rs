pub mod types {
    pub use gram_diagnostics::{Diagnostic, FileResult, LintResult, Position, Range, Severity};
}
pub mod lint;
pub(crate) mod rules;
pub(crate) mod markdown;
