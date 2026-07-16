//! Collect diagnostics from `typst-syntax` math parsing.

use typst_syntax::{LinkedNode, SyntaxKind, parse_math};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Suggestion};

/// Parse math and convert parser errors/warnings into diagnostics.
pub fn collect_syntax_diagnostics(text: &str) -> (typst_syntax::SyntaxNode, Vec<Diagnostic>) {
    let root = parse_math(text);
    let (errors, warnings) = root.errors_and_warnings();
    let ranges = collect_error_ranges(LinkedNode::new(&root));

    let mut diagnostics = Vec::new();

    for (i, err) in errors.into_iter().enumerate() {
        let span = ranges
            .get(i)
            .cloned()
            .unwrap_or(0..text.len().min(1));
        let span = clamp_span(span, text.len());
        let mut diag = Diagnostic::new(
            Severity::Error,
            DiagnosticCode::SyntaxParse,
            err.message.to_string(),
            span,
        );
        for hint in err.hints {
            diag.suggestions
                .push(Suggestion::new(hint.v.to_string(), None));
        }
        diagnostics.push(diag);
    }

    // Parser warnings are uncommon in math, but forward them when present.
    for warn in warnings {
        let span = clamp_span(0..text.len().min(1), text.len());
        let mut diag = Diagnostic::new(
            Severity::Warning,
            DiagnosticCode::SyntaxParse,
            warn.message.to_string(),
            span,
        );
        for hint in warn.hints {
            diag.suggestions
                .push(Suggestion::new(hint.v.to_string(), None));
        }
        diagnostics.push(diag);
    }

    (root, diagnostics)
}

fn collect_error_ranges(node: LinkedNode<'_>) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    walk_errors(node, &mut ranges);
    ranges
}

fn walk_errors(node: LinkedNode<'_>, ranges: &mut Vec<std::ops::Range<usize>>) {
    if node.kind() == SyntaxKind::Error {
        ranges.push(node.range());
    }
    for child in node.children() {
        walk_errors(child, ranges);
    }
}

fn clamp_span(span: std::ops::Range<usize>, len: usize) -> std::ops::Range<usize> {
    let start = span.start.min(len);
    let end = span.end.min(len).max(start);
    // Prefer a non-empty span when possible for easier UI highlighting.
    if start == end && start > 0 {
        (start - 1)..start
    } else if start == end && len > 0 {
        0..1
    } else {
        start..end
    }
}
