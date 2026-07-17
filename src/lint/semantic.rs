//! Light semantic hints for multi-letter identifiers and unknown symbols.

use typst_syntax::{LinkedNode, SyntaxKind};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Suggestion};
use crate::lint::names::{
    closest_symbol, is_known_function, is_known_ident, LATEX_IDENT_ALIASES,
};

/// Lint multi-letter idents, skipping spans already covered by higher-priority lints.
pub fn lint_semantic(
    root: &typst_syntax::SyntaxNode,
    occupied: &[std::ops::Range<usize>],
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    walk(LinkedNode::new(root), occupied, &mut out);
    out
}

fn walk(node: LinkedNode<'_>, occupied: &[std::ops::Range<usize>], out: &mut Vec<Diagnostic>) {
    if node.kind() == SyntaxKind::MathIdent {
        maybe_hint_ident(&node, occupied, out);
    }

    let is_call = node.kind() == SyntaxKind::MathCall;
    for child in node.children() {
        if is_call {
            // Skip the callee; name/latex lints already cover it.
            let is_callee = matches!(
                child.kind(),
                SyntaxKind::MathIdent | SyntaxKind::MathFieldAccess
            );
            if is_callee {
                continue;
            }
        }
        walk(child, occupied, out);
    }
}

fn maybe_hint_ident(
    node: &LinkedNode<'_>,
    occupied: &[std::ops::Range<usize>],
    out: &mut Vec<Diagnostic>,
) {
    let name = node.leaf_text().as_str();
    let span = node.range();

    if ranges_overlap(&span, occupied)
        || name.chars().count() <= 1
        || is_known_ident(name)
        || LATEX_IDENT_ALIASES.iter().any(|a| *a == name)
    {
        return;
    }

    if let Some(suggestion) = closest_symbol(name, 1)
        && suggestion != name
    {
        out.push(
            Diagnostic::new(
                Severity::Hint,
                DiagnosticCode::SemanticUnknownSymbol,
                format!("unknown symbol `{name}`; did you mean `{suggestion}`?"),
                span,
            )
            .with_suggestion(Suggestion::new(
                format!("use `{suggestion}`"),
                Some(suggestion.to_string()),
            )),
        );
        return;
    }

    if !is_known_function(name) {
        let quoted = format!("\"{name}\"");
        out.push(
            Diagnostic::new(
                Severity::Hint,
                DiagnosticCode::SemanticMultiLetterIdent,
                format!(
                    "multi-letter identifier `{name}` is treated as a single variable; \
                     quote it to display the letters literally"
                ),
                span,
            )
            .with_suggestion(Suggestion::new(
                format!("use `{quoted}` for literal text"),
                Some(quoted),
            )),
        );
    }
}

fn ranges_overlap(a: &std::ops::Range<usize>, occupied: &[std::ops::Range<usize>]) -> bool {
    occupied.iter().any(|b| a.start < b.end && b.start < a.end)
}
