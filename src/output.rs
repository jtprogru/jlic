//! Writing the result to disk.

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;

use crate::error::{Error, Result};

/// Writes text to a file.
///
/// An existing file is only overwritten with `force`, so an accidental `jlic`
/// in someone else's repository cannot clobber the license already chosen there.
///
/// The file is always created with `O_CREAT | O_EXCL`: the check and the write
/// are one operation, and a symlink at `path` is never followed — a repository
/// cannot redirect the write elsewhere by shipping `LICENSE` as a dangling link.
pub fn write(path: &Path, contents: &str, force: bool) -> Result<()> {
    let fail = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };

    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|source| Error::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    // Unlink instead of truncating, so `--force` replaces a symlink itself
    // rather than writing through it.
    if force {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(fail(source)),
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| match source.kind() {
            io::ErrorKind::AlreadyExists => Error::OutputExists(path.to_path_buf()),
            _ => fail(source),
        })?;

    file.write_all(contents.as_bytes()).map_err(fail)
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

    /// A repository shipping `LICENSE` as a symlink must not be able to make
    /// `jlic` write outside the directory it was pointed at.
    #[test]
    #[cfg(unix)]
    fn does_not_write_through_a_symlink() {
        let dir = std::env::temp_dir().join("jlic-symlink-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let victim = dir.join("victim");
        let link = dir.join("LICENSE");
        std::os::unix::fs::symlink(&victim, &link).unwrap();

        // The link dangles, so an `exists()` check would have said "free".
        assert!(matches!(
            write(&link, "text", false),
            Err(Error::OutputExists(_))
        ));
        assert!(!victim.exists(), "the write followed the symlink");

        // --force replaces the link, it does not write through it.
        write(&link, "text", true).unwrap();
        assert!(!victim.exists(), "--force followed the symlink");
        assert!(fs::symlink_metadata(&link).unwrap().is_file());

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
