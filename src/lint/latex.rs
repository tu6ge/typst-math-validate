//! LaTeX / common-alias → Typst math suggestions.

use std::ops::Range;

use typst_syntax::{LinkedNode, SyntaxKind};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Suggestion};

struct Alias {
    /// Source form as it appears in input (without leading `\`).
    from: &'static str,
    /// Typst replacement identifier or short form.
    to: &'static str,
    /// Extra guidance shown in the suggestion message.
    note: &'static str,
}

/// Identifier aliases (no backslash), typically LaTeX environment/command names.
const IDENT_ALIASES: &[Alias] = &[
    Alias {
        from: "matrix",
        to: "mat",
        note: "Typst uses `mat` for matrices",
    },
    Alias {
        from: "pmatrix",
        to: "mat",
        note: "Typst uses `mat`; set `delim: \"(\"` for parentheses",
    },
    Alias {
        from: "bmatrix",
        to: "mat",
        note: "Typst uses `mat`; set `delim: \"[\"` for brackets",
    },
    Alias {
        from: "vmatrix",
        to: "mat",
        note: "Typst uses `mat(delim: \"|\", ...)` for determinants",
    },
    Alias {
        from: "Vmatrix",
        to: "mat",
        note: "Typst uses `mat(delim: \"||\", ...)` for norms-style matrices",
    },
    Alias {
        from: "lg",
        to: "log",
        note: "Typst uses `log` (or `log` with a base) instead of `lg`",
    },
    Alias {
        from: "iint",
        to: "integral.double",
        note: "Typst uses `integral.double` for ∬ instead of `iint`",
    },
    Alias {
        from: "iiint",
        to: "integral.triple",
        note: "Typst uses `integral.triple` for ∭ instead of `iiint`",
    },
    Alias {
        from: "oint",
        to: "integral.cont",
        note: "Typst uses `integral.cont` for ∮ instead of `oint`",
    },
];

/// Backslash commands: `\from` → Typst `to`.
const COMMAND_ALIASES: &[Alias] = &[
    Alias {
        from: "frac",
        to: "frac",
        note: "use `frac(a, b)` or `a/b` instead of `\\frac{a}{b}`",
    },
    Alias {
        from: "sqrt",
        to: "sqrt",
        note: "use `sqrt(x)` instead of `\\sqrt{x}`",
    },
    Alias {
        from: "binom",
        to: "binom",
        note: "use `binom(n, k)` instead of `\\binom{n}{k}`",
    },
    Alias {
        from: "left",
        to: "lr",
        note: "use `lr` (or matching delimiters) instead of `\\left`",
    },
    Alias {
        from: "right",
        to: "lr",
        note: "use `lr` (or matching delimiters) instead of `\\right`",
    },
    Alias {
        from: "vec",
        to: "vec",
        note: "use `vec(...)` or `arrow` accents instead of `\\vec`",
    },
    Alias {
        from: "sum",
        to: "sum",
        note: "use `sum` without a backslash",
    },
    Alias {
        from: "prod",
        to: "product",
        note: "use `product` instead of `\\prod`",
    },
    Alias {
        from: "lim",
        to: "lim",
        note: "use `lim` without a backslash",
    },
    Alias {
        from: "infty",
        to: "infty",
        note: "use `infty` without a backslash",
    },
    Alias {
        from: "cdot",
        to: "dot",
        note: "use `dot` instead of `\\cdot`",
    },
    Alias {
        from: "times",
        to: "times",
        note: "use `times` without a backslash",
    },
    Alias {
        from: "alpha",
        to: "alpha",
        note: "use `alpha` without a backslash",
    },
    Alias {
        from: "beta",
        to: "beta",
        note: "use `beta` without a backslash",
    },
    Alias {
        from: "gamma",
        to: "gamma",
        note: "use `gamma` without a backslash",
    },
    Alias {
        from: "delta",
        to: "delta",
        note: "use `delta` without a backslash",
    },
    Alias {
        from: "theta",
        to: "theta",
        note: "use `theta` without a backslash",
    },
    Alias {
        from: "lambda",
        to: "lambda",
        note: "use `lambda` without a backslash",
    },
    Alias {
        from: "pi",
        to: "pi",
        note: "use `pi` without a backslash",
    },
    Alias {
        from: "ln",
        to: "ln",
        note: "use `ln` without a backslash",
    },
    Alias {
        from: "log",
        to: "log",
        note: "use `log` without a backslash",
    },
    Alias {
        from: "sin",
        to: "sin",
        note: "use `sin` without a backslash",
    },
    Alias {
        from: "cos",
        to: "cos",
        note: "use `cos` without a backslash",
    },
    Alias {
        from: "tan",
        to: "tan",
        note: "use `tan` without a backslash",
    },
    Alias {
        from: "cot",
        to: "cot",
        note: "use `cot` without a backslash",
    },
    Alias {
        from: "sec",
        to: "sec",
        note: "use `sec` without a backslash",
    },
    Alias {
        from: "csc",
        to: "csc",
        note: "use `csc` without a backslash",
    },
    Alias {
        from: "exp",
        to: "exp",
        note: "use `exp` without a backslash",
    },
    Alias {
        from: "iint",
        to: "integral.double",
        note: "use `integral.double` instead of `\\iint`",
    },
    Alias {
        from: "iiint",
        to: "integral.triple",
        note: "use `integral.triple` instead of `\\iiint`",
    },
    Alias {
        from: "oint",
        to: "integral.cont",
        note: "use `integral.cont` instead of `\\oint`",
    },
    Alias {
        from: "sigma",
        to: "sigma",
        note: "use `sigma` without a backslash",
    },
    Alias {
        from: "omega",
        to: "omega",
        note: "use `omega` without a backslash",
    },
];

/// Run LaTeX / alias lint over source text and the parse tree.
pub fn lint_latex(text: &str, root: &typst_syntax::SyntaxNode) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    scan_backslash_commands(text, &mut out);
    scan_begin_end(text, &mut out);
    walk_idents(LinkedNode::new(root), &mut out);
    out
}

fn walk_idents(node: LinkedNode<'_>, out: &mut Vec<Diagnostic>) {
    // Match both call style (`matrix(...)`) and attach style (`iint_D`).
    if node.kind() == SyntaxKind::MathIdent {
        let name = node.leaf_text().as_str();
        if let Some(alias) = IDENT_ALIASES.iter().find(|a| a.from == name) {
            out.push(alias_diagnostic(alias, node.range(), false));
        }
    }

    for child in node.children() {
        walk_idents(child, out);
    }
}

fn scan_backslash_commands(text: &str, out: &mut Vec<Diagnostic>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let start = i;
            i += 1;
            let name_start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphabetic()) {
                i += 1;
            }
            if i > name_start {
                let name = &text[name_start..i];
                if let Some(alias) = COMMAND_ALIASES.iter().find(|a| a.from == name) {
                    out.push(alias_diagnostic(alias, start..i, true));
                }
            }
            continue;
        }
        i += 1;
    }
}

fn scan_begin_end(text: &str, out: &mut Vec<Diagnostic>) {
    for (pat, env, replacement, note) in [
        (
            "\\begin{cases}",
            "cases",
            "cases",
            "use `cases(...)` instead of `\\begin{cases}...\\end{cases}`",
        ),
        (
            "\\end{cases}",
            "cases",
            "cases",
            "remove `\\end{cases}`; Typst uses `cases(...)`",
        ),
        (
            "\\begin{matrix}",
            "matrix",
            "mat",
            "use `mat(...)` instead of a `matrix` environment",
        ),
        (
            "\\end{matrix}",
            "matrix",
            "mat",
            "remove `\\end{matrix}`; Typst uses `mat(...)`",
        ),
        (
            "\\begin{pmatrix}",
            "pmatrix",
            "mat",
            "use `mat(delim: \"(\", ...)` instead of `pmatrix`",
        ),
        (
            "\\end{pmatrix}",
            "pmatrix",
            "mat",
            "remove `\\end{pmatrix}`; Typst uses `mat(...)`",
        ),
    ] {
        let mut search = 0;
        while let Some(rel) = text[search..].find(pat) {
            let start = search + rel;
            let end = start + pat.len();
            let _ = env;
            out.push(
                Diagnostic::new(
                    Severity::Warning,
                    DiagnosticCode::LatexAlias,
                    format!("LaTeX-style `{pat}` is not valid Typst math"),
                    start..end,
                )
                .with_suggestion(Suggestion::new(note, Some(replacement.to_string()))),
            );
            search = end;
        }
    }
}

fn alias_diagnostic(alias: &Alias, span: Range<usize>, from_command: bool) -> Diagnostic {
    let shown = if from_command {
        format!("\\{}", alias.from)
    } else {
        alias.from.to_string()
    };
    Diagnostic::new(
        Severity::Warning,
        DiagnosticCode::LatexAlias,
        format!("`{shown}` looks like LaTeX; Typst uses `{}`", alias.to),
        span,
    )
    .with_suggestion(Suggestion::new(alias.note, Some(alias.to.to_string())))
}
