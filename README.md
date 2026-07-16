# typst-math-validate

Validate Typst math markup and return actionable, English diagnostics with fix suggestions.

Useful when users mix LaTeX habits with Typst (for example writing `matrix` instead of [`mat`](https://typst.app/docs/reference/math/mat/)), mistype function names, or leave incomplete attachments like `x^`.

## Install

```toml
[dependencies]
typst-math-validate = { git = "https://github.com/tu6ge/typst-math-validate", branch = "master" } # or crates.io version when published
```

## Usage

```rust
use typst_math_validate::{validate, DiagnosticCode};

fn main() {
    let report = validate("matrix(1, 2; 3, 4)");

    if !report.is_ok() {
        eprintln!("syntax errors present");
    }

    for diag in &report.diagnostics {
        println!(
            "[{}] {}: {} @ {:?}",
            diag.code, diag.severity, diag.message, diag.span
        );
        for suggestion in &diag.suggestions {
            if let Some(repl) = &suggestion.replacement {
                println!("  suggestion: {} → `{repl}`", suggestion.message);
            } else {
                println!("  hint: {}", suggestion.message);
            }
        }
    }

    assert!(report.diagnostics.iter().any(|d| d.code == DiagnosticCode::LatexAlias));
}
```

### Input form

Pass either:

- a math body: `x^2 + mat(1, 2; 3, 4)`
- or a full equation wrapper: `$ x^2 $` (outer `$...$` is stripped)

Spans are byte ranges in the **normalized** math text (after `$` stripping).

## What it checks

| Kind | Code | Example |
|------|------|---------|
| Syntax errors from `typst-syntax` | `syntax.parse` | `x^` |
| LaTeX / common aliases | `latex.alias` | `matrix` → `mat`, `\frac` → `frac` |
| Function typos | `name.did_you_mean` | `matr` → `mat` |
| Unknown calls | `name.unknown_function` | `foobarbaz(...)` |
| Multi-letter idents | `semantic.multi_letter_ident` | `area` → `"area"` |
| Symbol typos | `semantic.unknown_symbol` | near-miss symbol names |

Built on [`typst-syntax::parse_math`](https://docs.rs/typst-syntax/latest/typst_syntax/fn.parse_math.html) and the [Typst math reference](https://typst.app/docs/reference/math/).

## License

See repository license files when present.
