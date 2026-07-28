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

    // Field modifiers like `double` in `integral.double` are not free idents.
    if node
        .parent()
        .is_some_and(|p| p.kind() == SyntaxKind::MathFieldAccess)
    {
        return;
    }

    if ranges_overlap(&span, occupied)
        || name.chars().count() <= 1
        || is_known_ident(name)
        || LATEX_IDENT_ALIASES.iter().any(|a| eq_latex_alias(a, name))
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
        if is_concatenated_uppercase_names(name) {
            let separated = name
                .chars()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            out.push(
                Diagnostic::new(
                    Severity::Warning,
                    DiagnosticCode::SemanticMultiLetterIdent,
                    format!(
                        "consecutive capital letters `{name}` are treated as one variable; \
                         write them separately if they are distinct names (e.g. points or factors)"
                    ),
                    span,
                )
                .with_suggestion(Suggestion::new(
                    format!("use `{separated}` for separate names"),
                    Some(separated),
                ))
                .with_suggestion(Suggestion::new(
                    format!("use `{quoted}` for one literal label"),
                    Some(quoted),
                )),
            );
            return;
        }
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

/// Whether an identifier looks like adjacent single-letter point names.
fn is_concatenated_uppercase_names(name: &str) -> bool {
    name.chars().count() > 1 && name.chars().all(|c| c.is_ascii_uppercase())
}

fn eq_latex_alias(known: &str, name: &str) -> bool {
    known == name || (known.is_ascii() && name.is_ascii() && known.eq_ignore_ascii_case(name))
}

fn ranges_overlap(a: &std::ops::Range<usize>, occupied: &[std::ops::Range<usize>]) -> bool {
    occupied.iter().any(|b| a.start < b.end && b.start < a.end)
}
