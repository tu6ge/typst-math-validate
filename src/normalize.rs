//! Normalize math input by stripping a surrounding `$...$` pair.

use std::borrow::Cow;

/// Normalized math text ready for parsing and linting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized<'a> {
    /// Math body after optional `$` stripping.
    pub text: Cow<'a, str>,
    /// Byte offset of `text` within the original input.
    pub offset_in_original: usize,
}

/// Strip a single pair of enclosing `$...$` delimiters when present.
///
/// Only the outermost wrapping dollars on the trimmed input are removed.
/// Inner content (including spaces) is preserved so Typst block/inline
/// semantics stay intact when the caller re-wraps the expression.
pub fn normalize(input: &str) -> Normalized<'_> {
    let trim_start = input.len() - input.trim_start().len();
    let trim_end = input.len() - input.trim_end().len();
    let trimmed = &input[trim_start..input.len() - trim_end];

    if trimmed.len() >= 2 && trimmed.starts_with('$') && trimmed.ends_with('$') {
        let inner = &trimmed[1..trimmed.len() - 1];
        Normalized {
            text: Cow::Borrowed(inner),
            offset_in_original: trim_start + 1,
        }
    } else {
        Normalized {
            text: Cow::Borrowed(trimmed),
            offset_in_original: trim_start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_dollars() {
        let n = normalize("$ x^2 $");
        assert_eq!(n.text.as_ref(), " x^2 ");
        assert_eq!(n.offset_in_original, 1);
    }

    #[test]
    fn leaves_bare_math() {
        let n = normalize("x^2");
        assert_eq!(n.text.as_ref(), "x^2");
        assert_eq!(n.offset_in_original, 0);
    }
}
