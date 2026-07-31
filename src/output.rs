//! Writing the result to disk.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

/// Writes text to a file.
///
/// An existing file is only overwritten with `force`, so an accidental `jlic`
/// in someone else's repository cannot clobber the license already chosen there.
pub fn write(path: &Path, contents: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(Error::OutputExists(path.to_path_buf()));
    }

    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|source| Error::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(path, contents).map_err(|source| Error::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_to_overwrite_without_force() {
        let dir = std::env::temp_dir().join("jlic-output-test");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("LICENSE");

        write(&path, "first", false).unwrap();
        assert!(matches!(
            write(&path, "second", false),
            Err(Error::OutputExists(_))
        ));
        write(&path, "second", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn creates_missing_parent_directories() {
        let dir = std::env::temp_dir().join("jlic-nested-test");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("a").join("b").join("LICENSE");

        write(&path, "text", false).unwrap();
        assert!(path.exists());

        fs::remove_dir_all(&dir).unwrap();
    }
}
