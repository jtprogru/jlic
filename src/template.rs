//! Minimal template engine: substitutes `{{year}}`, `{{holder}}`, `{{name}}`
//! and `{{email}}`.
//!
//! Custom markers are used instead of the square brackets from
//! choosealicense.com because square brackets appear inside the GPL and Apache
//! texts as part of the license itself.

use crate::context::Context;
use crate::error::{Error, Result};

pub(crate) const OPEN: &str = "{{";
const CLOSE: &str = "}}";

/// Substitutes context values into a template.
///
/// The `holder` value is computed lazily, so templates without a copyright
/// line render even when the name could not be determined.
pub fn render(template: &str, ctx: &Context) -> Result<String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    let mut holder: Option<String> = None;

    while let Some(start) = rest.find(OPEN) {
        let after_open = &rest[start + OPEN.len()..];
        let Some(end) = after_open.find(CLOSE) else {
            break;
        };
        let key = after_open[..end].trim();

        let value = match key {
            "year" => ctx.year.clone(),
            "holder" => match &holder {
                Some(value) => value.clone(),
                None => {
                    let value = ctx.holder()?;
                    holder = Some(value.clone());
                    value
                }
            },
            "name" => ctx.name.clone().ok_or(Error::MissingName)?,
            "email" => ctx.email.clone().unwrap_or_default(),
            // Not one of ours — leave the marker untouched.
            _ => {
                let consumed = start + OPEN.len() + end + CLOSE.len();
                out.push_str(&rest[..consumed]);
                rest = &rest[consumed..];
                continue;
            }
        };

        out.push_str(&rest[..start]);
        out.push_str(&value);
        rest = &after_open[end + CLOSE.len()..];
    }

    out.push_str(rest);
    Ok(out)
}

/// Verifies no placeholder of ours survived into the finished text.
pub fn ensure_resolved(rendered: &str) -> Result<()> {
    let leftovers: Vec<&str> = rendered
        .match_indices(OPEN)
        .filter_map(|(i, _)| {
            let tail = &rendered[i + OPEN.len()..];
            tail.find(CLOSE).map(|end| tail[..end].trim())
        })
        .filter(|key| matches!(*key, "year" | "holder" | "name" | "email"))
        .collect();

    if leftovers.is_empty() {
        Ok(())
    } else {
        Err(Error::UnresolvedPlaceholders(leftovers.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context {
            year: "2026".into(),
            name: Some("Mikhail Savin".into()),
            email: Some("jtprogru@gmail.com".into()),
        }
    }

    #[test]
    fn substitutes_known_keys() {
        let out = render("(c) {{year}} {{holder}} / {{name}} / {{email}}", &ctx()).unwrap();
        assert_eq!(
            out,
            "(c) 2026 Mikhail Savin <jtprogru@gmail.com> / Mikhail Savin / jtprogru@gmail.com"
        );
    }

    #[test]
    fn tolerates_whitespace_inside_markers() {
        assert_eq!(render("{{ year }}", &ctx()).unwrap(), "2026");
    }

    #[test]
    fn leaves_unknown_markers_untouched() {
        let out = render("{{program}} {{year}}", &ctx()).unwrap();
        assert_eq!(out, "{{program}} 2026");
        // A foreign marker is not an unresolved placeholder.
        assert!(ensure_resolved(&out).is_ok());
    }

    #[test]
    fn leaves_unterminated_marker_untouched() {
        let out = render("trailing {{year", &ctx()).unwrap();
        assert_eq!(out, "trailing {{year");
    }

    #[test]
    fn renders_without_name_when_holder_not_needed() {
        let ctx = Context {
            year: "2026".into(),
            name: None,
            email: None,
        };
        assert_eq!(render("year {{year}}", &ctx).unwrap(), "year 2026");
        assert!(matches!(
            render("{{holder}}", &ctx),
            Err(Error::MissingName)
        ));
    }

    #[test]
    fn ensure_resolved_reports_leftovers() {
        let err = ensure_resolved("Copyright (c) {{year}}").unwrap_err();
        assert!(matches!(err, Error::UnresolvedPlaceholders(keys) if keys == "year"));
    }
}
