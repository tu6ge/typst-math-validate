//! Known Typst math function and symbol names, plus edit-distance helpers.

use typst_syntax::{LinkedNode, SyntaxKind};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity, Suggestion};

/// Built-in / commonly used math functions available in equations.
pub const MATH_FUNCTIONS: &[&str] = &[
    "accent",
    "attach",
    "binom",
    "cancel",
    "cases",
    "class",
    "equation",
    "frac",
    "lr",
    "mat",
    "primes",
    "sqrt",
    "root",
    "display",
    "inline",
    "script",
    "sscript",
    "stretch",
    "op",
    "underline",
    "overline",
    "underbrace",
    "overbrace",
    "underbracket",
    "overbracket",
    "underparen",
    "overparen",
    "undershell",
    "overshell",
    "serif",
    "sans",
    "cal",
    "frak",
    "mono",
    "bb",
    "vec",
    "abs",
    "norm",
    "round",
    "ceil",
    "floor",
    // Common operators / named functions (also valid as Typst math symbols).
    "sin",
    "cos",
    "tan",
    "cot",
    "sec",
    "csc",
    "arcsin",
    "arccos",
    "arctan",
    "sinh",
    "cosh",
    "tanh",
    "log",
    "ln",
    "exp",
    "lim",
    "max",
    "min",
    "inf",
    "sup",
    "det",
    "dim",
    "ker",
    "arg",
    "deg",
    "gcd",
    "lcm",
    "mod",
    "sum",
    "product",
    "integral",
];

/// Common math symbols / operators that are valid multi-letter idents.
pub const MATH_SYMBOLS: &[&str] = &[
    "pi",
    "alpha",
    "beta",
    "gamma",
    "delta",
    "epsilon",
    "zeta",
    "eta",
    "theta",
    "iota",
    "kappa",
    "lambda",
    "mu",
    "nu",
    "xi",
    "rho",
    "sigma",
    "tau",
    "upsilon",
    "phi",
    "chi",
    "psi",
    "omega",
    "Gamma",
    "Delta",
    "Theta",
    "Lambda",
    "Xi",
    "Pi",
    "Sigma",
    "Phi",
    "Psi",
    "Omega",
    "RR",
    "NN",
    "ZZ",
    "QQ",
    "CC",
    "HH",
    "dot",
    "times",
    "div",
    "pm",
    "mp",
    "infty",
    "sum",
    "product",
    "integral",
    "lim",
    "max",
    "min",
    "inf",
    "sup",
    "sin",
    "cos",
    "tan",
    "cot",
    "sec",
    "csc",
    "arcsin",
    "arccos",
    "arctan",
    "sinh",
    "cosh",
    "tanh",
    "log",
    "ln",
    "exp",
    "det",
    "dim",
    "ker",
    "arg",
    "deg",
    "gcd",
    "lcm",
    "mod",
    "diff",
    "dif",
    "grad",
    "curl",
    "laplacian",
    "partial",
    "nabla",
    "emptyset",
    "nothing",
    "infinity",
    "subset",
    "supset",
    "union",
    "sect",
    "forall",
    "exists",
    "bot",
    "top",
    "perp",
    "parallel",
    "angle",
    "ell",
    "aleph",
    "hbar",
    "Re",
    "Im",
    "approx",
    "equiv",
    "prec",
    "succ",
    "arrow",
    "plus",
    "minus",
    "ast",
    "star",
    "circ",
    "bullet",
];

/// Names that are intentional LaTeX aliases and handled by the latex lint.
pub const LATEX_IDENT_ALIASES: &[&str] = &[
    "matrix", "pmatrix", "bmatrix", "vmatrix", "Vmatrix", "lg",
];

/// Whether `name` is a known math function.
pub fn is_known_function(name: &str) -> bool {
    MATH_FUNCTIONS.iter().any(|s| eq_ident(s, name))
}

/// Whether `name` is a known symbol or function (safe multi-letter ident).
pub fn is_known_ident(name: &str) -> bool {
    is_known_function(name)
        || MATH_SYMBOLS.iter().any(|s| eq_ident(s, name))
        || LATEX_IDENT_ALIASES.iter().any(|s| eq_ident(s, name))
}

/// Case-insensitive match for ASCII operator names (`Ln` ≈ `ln`),
/// exact match otherwise (so `RR` stays distinct from `rr`).
fn eq_ident(known: &str, name: &str) -> bool {
    if known == name {
        return true;
    }
    // Only fold purely ASCII lowercase operator-style names.
    known.is_ascii()
        && name.is_ascii()
        && known.bytes().all(|b| b.is_ascii_lowercase())
        && known.eq_ignore_ascii_case(name)
}

/// Find the closest known function within `max_dist` edit distance.
pub fn closest_function(name: &str, max_dist: usize) -> Option<&'static str> {
    closest_in(name, MATH_FUNCTIONS, max_dist)
}

/// Find the closest known symbol within `max_dist` edit distance.
pub fn closest_symbol(name: &str, max_dist: usize) -> Option<&'static str> {
    closest_in(name, MATH_SYMBOLS, max_dist)
}

fn closest_in(name: &str, candidates: &[&'static str], max_dist: usize) -> Option<&'static str> {
    let mut best: Option<(&'static str, usize)> = None;
    for &cand in candidates {
        let dist = edit_distance(name, cand);
        if dist == 0 {
            return Some(cand);
        }
        if dist <= max_dist {
            match best {
                Some((_, d)) if dist >= d => {}
                _ => best = Some((cand, dist)),
            }
        }
    }
    best.map(|(s, _)| s)
}

/// Classic Levenshtein distance.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1)
                .min(curr[j] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Lint unknown / misspelled math function calls.
pub fn lint_names(root: &typst_syntax::SyntaxNode) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    walk(LinkedNode::new(root), &mut out);
    out
}

fn walk(node: LinkedNode<'_>, out: &mut Vec<Diagnostic>) {
    // Typst allows both library functions (`mat`) and symbols/ops (`sin`, `lim`)
    // in call position, e.g. `sin(x)`. Treat known symbols as valid callees.
    if node.kind() == SyntaxKind::MathCall
        && let Some((name, span)) = call_callee_ident(&node)
        && !is_latex_ident_alias(&name)
        && !is_known_ident(&name)
    {
        if let Some(suggestion) = closest_function(&name, 2) {
            out.push(
                Diagnostic::new(
                    Severity::Warning,
                    DiagnosticCode::NameDidYouMean,
                    format!(
                        "unknown math function `{name}`; did you mean `{suggestion}`?"
                    ),
                    span,
                )
                .with_suggestion(Suggestion::new(
                    format!("use `{suggestion}`"),
                    Some(suggestion.to_string()),
                )),
            );
        } else {
            out.push(Diagnostic::new(
                Severity::Warning,
                DiagnosticCode::NameUnknownFunction,
                format!("unknown math function `{name}`"),
                span,
            ));
        }
    }

    for child in node.children() {
        walk(child, out);
    }
}

/// Whether `name` is handled by the LaTeX-alias lint instead.
fn is_latex_ident_alias(name: &str) -> bool {
    LATEX_IDENT_ALIASES.iter().any(|s| eq_ident(s, name))
}

/// Return the callee identifier text and span for a `MathCall` node.
fn call_callee_ident(call: &LinkedNode<'_>) -> Option<(String, std::ops::Range<usize>)> {
    for child in call.children() {
        match child.kind() {
            SyntaxKind::MathIdent => {
                let name = child.leaf_text().to_string();
                return Some((name, child.range()));
            }
            SyntaxKind::MathFieldAccess => {
                // Prefer the final field name (e.g. `math.mat` → `mat`).
                let mut last = None;
                for part in child.children() {
                    if part.kind() == SyntaxKind::MathIdent {
                        last = Some((part.leaf_text().to_string(), part.range()));
                    }
                }
                return last;
            }
            _ => {}
        }
    }
    None
}
