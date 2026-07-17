//! Diagnostic types for Typst math validation.

use std::fmt;
use std::ops::Range;

/// Severity of a diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// A hard problem that prevents correct parsing or usage.
    Error,
    /// Likely incorrect usage with a suggested fix.
    Warning,
    /// Soft guidance that may or may not indicate a mistake.
    Hint,
}

/// Stable, machine-readable diagnostic codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// Syntax error from the Typst math parser.
    SyntaxParse,
    /// LaTeX or common-alias usage that should be rewritten in Typst.
    LatexAlias,
    /// Call to an unknown math function with no close match.
    NameUnknownFunction,
    /// Likely misspelling of a known math function.
    NameDidYouMean,
    /// Multi-letter identifier that may need quoting.
    SemanticMultiLetterIdent,
    /// Multi-letter identifier that may be a misspelled symbol.
    SemanticUnknownSymbol,
}

impl DiagnosticCode {
    /// Dotted string form used in tooling (e.g. `latex.alias`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SyntaxParse => "syntax.parse",
            Self::LatexAlias => "latex.alias",
            Self::NameUnknownFunction => "name.unknown_function",
            Self::NameDidYouMean => "name.did_you_mean",
            Self::SemanticMultiLetterIdent => "semantic.multi_letter_ident",
            Self::SemanticUnknownSymbol => "semantic.unknown_symbol",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => f.write_str("error"),
            Self::Warning => f.write_str("warning"),
            Self::Hint => f.write_str("hint"),
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A concrete fix suggestion attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// Human-readable explanation of the suggestion.
    pub message: String,
    /// Optional replacement text for the highlighted span.
    pub replacement: Option<String>,
}

impl Suggestion {
    /// Create a suggestion with a message and optional replacement.
    pub fn new(message: impl Into<String>, replacement: Option<String>) -> Self {
        Self {
            message: message.into(),
            replacement,
        }
    }
}

/// A single validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// How serious the finding is.
    pub severity: Severity,
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// English description of the problem.
    pub message: String,
    /// Byte range in the normalized math text.
    pub span: Range<usize>,
    /// Optional fix suggestions.
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    pub(crate) fn new(
        severity: Severity,
        code: DiagnosticCode,
        message: impl Into<String>,
        span: Range<usize>,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            span,
            suggestions: Vec::new(),
        }
    }

    pub(crate) fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
}

/// Result of validating a math expression.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    /// Findings sorted by span start, then severity.
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    /// Returns `true` when there are no [`Severity::Error`] diagnostics.
    pub fn is_ok(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Returns `true` when any warning is present.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning)
    }

    /// Returns `true` when the report contains no diagnostics at all.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
