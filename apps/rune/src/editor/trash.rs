//! Move a file or folder to the system Trash, where it can be put back.

use std::path::Path;

#[cfg(target_os = "macos")]
pub fn move_to_trash(path: &Path) -> Result<(), String> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    // SAFETY: plain calls with an owned path string, a valid file URL, and
    // no result URL wanted.
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
        NSFileManager::defaultManager().trashItemAtURL_resultingItemURL_error(&url, None)
    }
    .map_err(|e| format!("cannot move to the Trash: {}", e.localizedDescription()))
}

/// Linux follows the freedesktop.org Trash specification: home trash at
/// `$XDG_DATA_HOME/Trash` (default `~/.local/share/Trash`), which desktop file
/// managers list and restore from.
#[cfg(target_os = "linux")]
pub fn move_to_trash(path: &Path) -> Result<(), String> {
    let data = data_home()?;
    move_to_trash_in(&data, path)
}

// ponytail: macOS and Linux only; other platforms refuse rather than delete for good.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn move_to_trash(_: &Path) -> Result<(), String> {
    Err("moving to the Trash is not supported on this platform".into())
}

/// The trash root's parent: `$XDG_DATA_HOME`, else `~/.local/share`. A relative
/// `XDG_DATA_HOME` is ignored, as the specification requires.
#[cfg(target_os = "linux")]
fn data_home() -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|d| d.is_absolute())
    {
        return Ok(dir);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|d| d.is_absolute())
        .ok_or_else(|| "cannot find HOME to locate the Trash".to_string())?;
    Ok(home.join(".local/share"))
}

/// Move `path` into the Trash rooted at `data` (an `$XDG_DATA_HOME`-shaped
/// directory). Split from [`move_to_trash`] so tests drive it directly.
#[cfg(target_os = "linux")]
fn move_to_trash_in(data: &Path, path: &Path) -> Result<(), String> {
    use std::fs;

    let source =
        std::path::absolute(path).map_err(|e| format!("cannot resolve {}: {e}", path.display()))?;
    if !source.exists() {
        return Err(format!("{} does not exist", source.display()));
    }
    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("{} has no usable file name", source.display()))?
        .to_string();

    let trash = data.join("Trash");
    let files = trash.join("files");
    let info = trash.join("info");
    for dir in [&files, &info] {
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }

    let (target, info_path) = unique_destination(&files, &info, &name);
    // Write the restore entry first: a moved file without it cannot be put
    // back, while an entry without a file is harmless and easy to remove.
    let stamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S");
    let entry = format!(
        "[Trash Info]\nPath={}\nDeletionDate={stamp}\n",
        encode_path(&source)
    );
    fs::write(&info_path, entry)
        .map_err(|e| format!("cannot write {}: {e}", info_path.display()))?;

    let moved = fs::rename(&source, &target).or_else(|_| {
        // A different filesystem (a checkout on a mount of its own): copy, then
        // drop the original, so the Trash stays one directory.
        copy_recursive(&source, &target)?;
        if source.is_dir() && !source.is_symlink() {
            fs::remove_dir_all(&source)
        } else {
            fs::remove_file(&source)
        }
    });
    if let Err(e) = moved {
        // Drop the restore entry and anything a partial copy left behind, so a
        // failed move never looks like a trashed file.
        let _ = fs::remove_file(&info_path);
        let _ = remove_path(&target);
        return Err(format!(
            "cannot move {} to {}: {e}",
            source.display(),
            target.display()
        ));
    }
    Ok(())
}

/// Remove `path` whatever it is: a symlink is unlinked, not followed.
#[cfg(target_os = "linux")]
fn remove_path(path: &Path) -> std::io::Result<()> {
    use std::fs;
    if path.is_symlink() || !path.is_dir() {
        return fs::remove_file(path);
    }
    fs::remove_dir_all(path)
}

/// `files/<name>` plus `info/<name>.trashinfo`, numbered `.2`, `.3`, …
/// (before the info suffix) while either side is taken.
#[cfg(target_os = "linux")]
fn unique_destination(
    files: &Path,
    info: &Path,
    name: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    for n in 1u32.. {
        let candidate = if n == 1 {
            name.to_string()
        } else {
            format!("{name}.{n}")
        };
        let file = files.join(&candidate);
        let entry = info.join(format!("{candidate}.trashinfo"));
        if !file.exists() && !entry.exists() {
            return (file, entry);
        }
    }
    unreachable!("the numbered loop always finds a free slot")
}

#[cfg(target_os = "linux")]
fn copy_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::fs;
    // A symlink is copied as a link, never followed: trashing a link must not
    // drag its target along.
    if from.is_symlink() {
        let target = fs::read_link(from)?;
        return std::os::unix::fs::symlink(target, to);
    }
    if from.is_dir() {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &to.join(entry.file_name()))?;
        }
        return Ok(());
    }
    fs::copy(from, to).map(|_| ())
}

/// Percent-encode the absolute path for the `Path=` field: only unreserved
/// ASCII and `/` stay literal, every other byte becomes `%XX`.
#[cfg(target_os = "linux")]
fn encode_path(path: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    let mut out = String::new();
    for &byte in path.as_os_str().as_bytes() {
        let literal =
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/');
        if literal {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::{encode_path, move_to_trash_in};
    use app_core::test_support::TempDir;
    use std::fs;
    use std::path::Path;

    fn entry(home: &Path, name: &str) -> String {
        fs::read_to_string(home.join("Trash/info").join(format!("{name}.trashinfo")))
            .expect("the restore entry exists")
    }

    #[test]
    fn a_file_lands_in_the_freedesktop_trash_with_a_restore_entry() {
        let home = TempDir::new("trash-home");
        let work = TempDir::new("trash-work");
        let source = work.path().join("abc.md");
        fs::write(&source, "hello").unwrap();

        move_to_trash_in(home.path(), &source).unwrap();

        assert!(!source.exists(), "the original is gone");
        assert_eq!(
            fs::read_to_string(home.path().join("Trash/files/abc.md")).unwrap(),
            "hello"
        );
        let info = entry(home.path(), "abc.md");
        assert!(info.starts_with("[Trash Info]\n"), "{info}");
        assert!(
            info.contains(&format!("Path={}", encode_path(&source))),
            "{info}"
        );
        assert!(info.contains("\nDeletionDate="), "{info}");
    }

    #[test]
    fn a_second_file_of_the_same_name_gets_a_numbered_entry() {
        let home = TempDir::new("trash-home-dup");
        for _ in 0..2 {
            let work = TempDir::new("trash-work-dup");
            fs::write(work.path().join("notes.txt"), "x").unwrap();
            move_to_trash_in(home.path(), &work.path().join("notes.txt")).unwrap();
        }
        assert!(home.path().join("Trash/files/notes.txt").exists());
        assert!(home.path().join("Trash/files/notes.txt.2").exists());
        assert!(
            home.path()
                .join("Trash/info/notes.txt.2.trashinfo")
                .exists()
        );
    }

    #[test]
    fn a_folder_moves_with_everything_under_it() {
        let home = TempDir::new("trash-home-dir");
        let work = TempDir::new("trash-work-dir");
        let dir = work.path().join("logs");
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("nested/a.txt"), "a").unwrap();

        move_to_trash_in(home.path(), &dir).unwrap();

        assert!(!dir.exists());
        assert_eq!(
            fs::read_to_string(home.path().join("Trash/files/logs/nested/a.txt")).unwrap(),
            "a"
        );
        assert!(home.path().join("Trash/info/logs.trashinfo").exists());
    }

    #[test]
    fn a_missing_path_is_refused_without_touching_the_trash() {
        let home = TempDir::new("trash-home-missing");
        let missing = home.path().join("nope.txt");
        assert!(move_to_trash_in(home.path(), &missing).is_err());
        assert!(!home.path().join("Trash").exists());
    }

    #[test]
    fn a_symlink_moves_as_a_link_and_keeps_its_target() {
        let home = TempDir::new("trash-home-link");
        let work = TempDir::new("trash-work-link");
        let target = work.path().join("real.txt");
        fs::write(&target, "keep").unwrap();
        let link = work.path().join("link.txt");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        move_to_trash_in(home.path(), &link).unwrap();

        assert!(!link.exists(), "the link is gone from the checkout");
        assert!(target.exists(), "the target stays");
        let trashed = home.path().join("Trash/files/link.txt");
        assert!(
            fs::symlink_metadata(&trashed)
                .unwrap()
                .file_type()
                .is_symlink(),
            "the trash holds a link, not a copy of the target"
        );
        assert_eq!(fs::read_link(&trashed).unwrap(), target);
    }

    #[test]
    fn encode_path_keeps_unreserved_bytes_and_escapes_the_rest() {
        assert_eq!(encode_path(Path::new("/tmp/a b/c.txt")), "/tmp/a%20b/c.txt");
        assert_eq!(encode_path(Path::new("/tmp/x_y-z~.rs")), "/tmp/x_y-z~.rs");
    }
}
