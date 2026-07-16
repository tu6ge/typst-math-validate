//! Lint passes on top of the Typst math syntax tree.

mod latex;
mod names;
mod semantic;

pub use latex::lint_latex;
pub use names::lint_names;
pub use semantic::lint_semantic;

use crate::diagnostic::Diagnostic;

/// Run all lint passes and return combined diagnostics.
pub fn run_lints(text: &str, root: &typst_syntax::SyntaxNode) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(lint_latex(text, root));
    diagnostics.extend(lint_names(root));

    let occupied: Vec<_> = diagnostics.iter().map(|d| d.span.clone()).collect();
    diagnostics.extend(lint_semantic(root, &occupied));
    diagnostics
}
