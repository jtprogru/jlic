use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown license: {requested}{}", suggestion.map(|s| format!(" (did you mean {s}?)")).unwrap_or_default())]
    UnknownLicense {
        requested: String,
        suggestion: Option<&'static str>,
    },

    #[error(
        "cannot determine the copyright holder: pass --name, set JLIC_NAME, or configure git config user.name"
    )]
    MissingName,

    #[error("invalid year: {0} (expected 2026 or 2020-2026)")]
    InvalidYear(String),

    #[error("invalid {field}: {reason}")]
    InvalidHolderField {
        field: &'static str,
        reason: &'static str,
    },

    #[error("{0} already exists: overwrite it with --force or pick another path with --output")]
    OutputExists(PathBuf),

    #[error("cannot write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("the rendered template still contains unresolved placeholders: {0}")]
    UnresolvedPlaceholders(String),
}

pub type Result<T> = std::result::Result<T, Error>;
