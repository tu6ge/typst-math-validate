//! Validate Typst math markup and return actionable diagnostics.
//!
//! # Example
//!
//! ```
//! use typst_math_validate::validate;
//!
//! let report = validate("matrix(1, 2; 3, 4)");
//! assert!(!report.is_clean());
//! assert!(report.diagnostics.iter().any(|d| {
//!     d.suggestions.iter().any(|s| s.replacement.as_deref() == Some("mat"))
//! }));
//! ```

mod diagnostic;
mod lint;
mod normalize;
mod syntax;

pub use diagnostic::{
    Diagnostic, DiagnosticCode, Severity, Suggestion, ValidationReport,
};
pub use normalize::{normalize, Normalized};

use lint::run_lints;
use syntax::collect_syntax_diagnostics;

/// Validate a Typst math expression.
///
/// Accepts a math body such as `x^2 + 1`, or a whole equation wrapped in
/// `$...$` (the dollars are stripped automatically). Diagnostics use byte
/// spans relative to the normalized math text (after `$` stripping).
pub fn validate(input: &str) -> ValidationReport {
    let normalized = normalize(input);
    let text = normalized.text.as_ref();

    let (root, mut diagnostics) = collect_syntax_diagnostics(text);
    diagnostics.extend(run_lints(text, &root));

    diagnostics.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then_with(|| a.severity.cmp(&b.severity))
            .then_with(|| a.code.as_str().cmp(b.code.as_str()))
    });

    // Light de-duplication: same code + same span keeps the first.
    diagnostics.dedup_by(|a, b| a.code == b.code && a.span == b.span);

    ValidationReport { diagnostics }
}
