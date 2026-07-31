//! Values to substitute into templates: year, copyright holder, email.

use std::process::Command;

use time::OffsetDateTime;

use crate::error::{Error, Result};

/// Environment variables that take precedence over git config.
const ENV_NAME: &str = "JLIC_NAME";
const ENV_EMAIL: &str = "JLIC_EMAIL";

/// Substitution values for a template.
///
/// `name` is optional: licenses without a copyright line (GPL, MPL) render
/// without it, so there is no point in demanding a name up front.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub year: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl Context {
    /// Resolves the context: explicit arguments, then environment, then git config.
    pub fn resolve(year: Option<&str>, name: Option<&str>, email: Option<&str>) -> Result<Self> {
        let year = match year {
            Some(y) => validate_year(y)?,
            None => current_year(),
        };

        let name = first_non_empty([
            name.map(str::to_string),
            env_var(ENV_NAME),
            git_config("user.name"),
        ]);

        let email = first_non_empty([
            email.map(str::to_string),
            env_var(ENV_EMAIL),
            git_config("user.email"),
        ]);

        Ok(Self { year, name, email })
    }

    /// The holder string: `Name` or `Name <email>`.
    pub fn holder(&self) -> Result<String> {
        let name = self.name.as_deref().ok_or(Error::MissingName)?;
        Ok(match self.email.as_deref() {
            Some(email) => format!("{name} <{email}>"),
            None => name.to_string(),
        })
    }
}

fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

fn first_non_empty<const N: usize>(candidates: [Option<String>; N]) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .map(|v| v.trim().to_string())
        .find(|v| !v.is_empty())
}

/// Reads a value from git config. A missing git is not an error.
fn git_config(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn current_year() -> String {
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .year()
        .to_string()
}

/// Accepts `2026` or a range like `2020-2026`.
fn validate_year(raw: &str) -> Result<String> {
    let value = raw.trim();
    let invalid = || Error::InvalidYear(raw.to_string());

    let is_year = |s: &str| s.len() == 4 && s.chars().all(|c| c.is_ascii_digit());

    match value.split_once('-') {
        None if is_year(value) => Ok(value.to_string()),
        Some((from, to)) if is_year(from) && is_year(to) && from <= to => Ok(value.to_string()),
        _ => Err(invalid()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_single_year_and_range() {
        assert_eq!(validate_year("2026").unwrap(), "2026");
        assert_eq!(validate_year(" 2020-2026 ").unwrap(), "2020-2026");
    }

    #[test]
    fn rejects_malformed_years() {
        for bad in ["26", "20x6", "2026-2020", "2026-", "-2026", "", "2026 2027"] {
            assert!(validate_year(bad).is_err(), "should be rejected: {bad}");
        }
    }

    #[test]
    fn holder_includes_email_when_present() {
        let ctx = Context {
            year: "2026".into(),
            name: Some("Mikhail Savin".into()),
            email: Some("jtprogru@gmail.com".into()),
        };
        assert_eq!(ctx.holder().unwrap(), "Mikhail Savin <jtprogru@gmail.com>");

        let ctx = Context { email: None, ..ctx };
        assert_eq!(ctx.holder().unwrap(), "Mikhail Savin");
    }

    #[test]
    fn holder_without_name_is_an_error() {
        let ctx = Context {
            year: "2026".into(),
            name: None,
            email: None,
        };
        assert!(matches!(ctx.holder(), Err(Error::MissingName)));
    }

    #[test]
    fn explicit_values_win_over_environment() {
        let ctx = Context::resolve(Some("1999"), Some("Someone"), Some("a@b.c")).unwrap();
        assert_eq!(ctx.year, "1999");
        assert_eq!(ctx.name.as_deref(), Some("Someone"));
        assert_eq!(ctx.email.as_deref(), Some("a@b.c"));
    }

    #[test]
    fn current_year_is_four_digits() {
        let year = current_year();
        assert_eq!(year.len(), 4, "unexpected year: {year}");
    }
}
