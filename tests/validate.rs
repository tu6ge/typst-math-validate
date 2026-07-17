//! Integration tests for Typst math validation.

use typst_math_validate::{validate, DiagnosticCode, Severity};

fn codes(input: &str) -> Vec<DiagnosticCode> {
    validate(input)
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect()
}

fn has_replacement(input: &str, replacement: &str) -> bool {
    validate(input).diagnostics.iter().any(|d| {
        d.suggestions
            .iter()
            .any(|s| s.replacement.as_deref() == Some(replacement))
    })
}

#[test]
fn accepts_valid_math() {
    for input in [
        "x^2",
        "mat(1, 2; 3, 4)",
        "sum_(k=0)^n k",
        "frac(a^2, 2)",
        "vec(1, 2, delim: \"[\")",
        "pi r^2",
        "sin(x)",
        "cos(theta)",
        "log(x)",
        "lim_(x -> 0) f(x)",
        "$ x^2 $",
    ] {
        let report = validate(input);
        assert!(
            report.is_ok(),
            "expected ok for {input:?}, got {:?}",
            report.diagnostics
        );
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Warning),
            "unexpected warnings for {input:?}: {:?}",
            report.diagnostics
        );
    }
}

#[test]
fn strips_dollar_wrappers() {
    let bare = validate("x^2");
    let wrapped = validate("$x^2$");
    assert!(bare.is_ok());
    assert!(wrapped.is_ok());
}

#[test]
fn suggests_mat_for_matrix() {
    let report = validate("matrix(1, 2; 3, 4)");
    assert!(report.has_warnings());
    assert!(codes("matrix(1, 2; 3, 4)").contains(&DiagnosticCode::LatexAlias));
    assert!(has_replacement("matrix(1, 2; 3, 4)", "mat"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("mat")),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn suggests_mat_for_pmatrix() {
    assert!(has_replacement("pmatrix(1, 2)", "mat"));
    assert!(codes("pmatrix(1, 2)").contains(&DiagnosticCode::LatexAlias));
}

#[test]
fn latex_backslash_commands() {
    assert!(codes(r"\frac{a}{b}").contains(&DiagnosticCode::LatexAlias));
    assert!(has_replacement(r"\frac{a}{b}", "frac"));
    assert!(codes(r"\sqrt{x}").contains(&DiagnosticCode::LatexAlias));
    assert!(has_replacement(r"\sum_(i=0)^n i", "sum"));
    assert!(codes(r"\begin{cases}").contains(&DiagnosticCode::LatexAlias));
    assert!(has_replacement(r"\begin{cases}", "cases"));
}

#[test]
fn spelling_correction_for_matr() {
    let report = validate("matr(1, 2)");
    assert!(codes("matr(1, 2)").contains(&DiagnosticCode::NameDidYouMean));
    assert!(has_replacement("matr(1, 2)", "mat"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("did you mean")),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn spelling_correction_for_binoms() {
    assert!(has_replacement("binoms(n, k)", "binom"));
}

#[test]
fn unknown_function_without_close_match() {
    let report = validate("foobarbaz(1, 2)");
    assert!(codes("foobarbaz(1, 2)").contains(&DiagnosticCode::NameUnknownFunction));
    assert!(report.has_warnings());
}

#[test]
fn multi_letter_ident_hint() {
    let report = validate("area = pi r^2");
    assert!(codes("area = pi r^2").contains(&DiagnosticCode::SemanticMultiLetterIdent));
    assert!(has_replacement("area = pi r^2", "\"area\""));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Hint && d.message.contains("area")),
        "{:?}",
        report.diagnostics
    );
    // `pi` is a known symbol and should not be flagged.
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("`pi`")),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn syntax_error_on_incomplete_attach() {
    let report = validate("x^");
    assert!(!report.is_ok());
    assert!(codes("x^").contains(&DiagnosticCode::SyntaxParse));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn syntax_error_on_incomplete_frac() {
    let report = validate("a/");
    assert!(!report.is_ok());
    assert!(codes("a/").contains(&DiagnosticCode::SyntaxParse));
}

#[test]
fn suggests_infty_for_quoted_oo() {
    let input = r#"sum_(n=1)^"oo" 1/n^2"#;
    let report = validate(input);
    assert!(
        codes(input).contains(&DiagnosticCode::SemanticInfinityAlias),
        "{:?}",
        report.diagnostics
    );
    assert!(has_replacement(input, "infty"));
    assert!(
        report.diagnostics.iter().any(|d| {
            d.message.contains("infinity") && d.suggestions.iter().any(|s| {
                s.replacement.as_deref() == Some("infty")
            })
        }),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn suggests_infty_for_bare_oo() {
    let input = "sum_(n=1)^oo 1/n^2";
    assert!(codes(input).contains(&DiagnosticCode::SemanticInfinityAlias));
    assert!(has_replacement(input, "infty"));
}

#[test]
fn unknown_symbol_near_miss() {
    // `alph` is one edit from `alpha`.
    let report = validate("alph");
    assert!(codes("alph").contains(&DiagnosticCode::SemanticUnknownSymbol));
    assert!(has_replacement("alph", "alpha"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Hint),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn spans_are_within_normalized_text() {
    let input = "matrix(1,2)";
    let report = validate(input);
    for d in &report.diagnostics {
        assert!(d.span.start <= d.span.end);
        assert!(d.span.end <= input.len());
        assert_eq!(&input[d.span.clone()], "matrix");
    }
}
