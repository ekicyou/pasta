//! Stage 1.6: find and read the event table (`calendar.csv`) in `ghost/master`.

use std::fs;
use std::path::{Path, PathBuf};

use pasta_core::{CalendarTable, is_calendar_file_name};

use super::LoaderError;

/// Stage 1.6. No table file gives an empty table (pasta has no built-in table).
pub(crate) fn load(base_dir: &Path) -> Result<CalendarTable, LoaderError> {
    let mut paths = find(base_dir)?;
    let path = match paths.len() {
        0 => return Ok(CalendarTable::default()),
        1 => paths.pop().unwrap(),
        _ => {
            paths.sort();
            return Err(LoaderError::CalendarDuplicate { paths });
        }
    };
    let bytes = fs::read(&path).map_err(|e| LoaderError::io(&path, e))?;
    let text = String::from_utf8(bytes).map_err(|_| LoaderError::CalendarNotUtf8(path.clone()))?;
    CalendarTable::parse(&text).map_err(|errors| LoaderError::Calendar { path, errors })
}

/// List `base_dir` once (not asking the OS, so `Calendar.CSV` is found on
/// case-sensitive file systems too). Files and folders both count; symlinks
/// do not, because `pasta_check` and the release copy skip them.
fn find(base_dir: &Path) -> Result<Vec<PathBuf>, LoaderError> {
    let mut found = Vec::new();
    for entry in fs::read_dir(base_dir).map_err(|e| LoaderError::io(base_dir, e))? {
        let entry = entry.map_err(|e| LoaderError::io(base_dir, e))?;
        let is_symlink = entry
            .file_type()
            .map_err(|e| LoaderError::io(entry.path(), e))?
            .is_symlink();
        if !is_symlink && is_calendar_file_name(&entry.file_name().to_string_lossy()) {
            found.push(entry.path());
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ghost() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn no_table_gives_empty_table() {
        let dir = ghost();
        fs::write(dir.path().join("pasta.toml"), "").unwrap();
        assert_eq!(load(dir.path()).unwrap(), CalendarTable::default());
    }

    #[test]
    fn reads_table_with_any_case() {
        let dir = ghost();
        fs::write(dir.path().join("Calendar.CSV"), "0101,元日の行事\n").unwrap();
        let table = load(dir.path()).unwrap();
        assert_eq!(table.event_name(2026, 1, 1), Some("元日の行事"));
    }

    #[test]
    fn line_errors_become_calendar_error_with_path_and_lines() {
        let dir = ghost();
        let path = dir.path().join("calendar.csv");
        fs::write(&path, "0101,ok\n13xx,bad\n0102\n").unwrap();
        let err = load(dir.path()).unwrap_err();
        let msg = err.to_string();
        assert!(
            matches!(&err, LoaderError::Calendar { errors, .. } if errors.len() == 2),
            "{msg}"
        );
        assert!(msg.contains(&path.display().to_string()), "{msg}");
        assert!(
            msg.contains("line 2: ") && msg.contains("; line 3: "),
            "{msg}"
        );
        assert!(!msg.contains('\n'), "{msg}");
    }

    #[test]
    fn not_utf8_is_encoding_error() {
        let dir = ghost();
        let path = dir.path().join("calendar.csv");
        // "0101,元日" in Shift_JIS
        fs::write(&path, b"0101,\x8c\xb3\x93\xfa\n").unwrap();
        let err = load(dir.path()).unwrap_err();
        assert!(matches!(&err, LoaderError::CalendarNotUtf8(p) if *p == path));
        let msg = err.to_string();
        assert!(msg.contains("UTF-8") && msg.contains(&path.display().to_string()));
    }

    #[test]
    fn folder_named_calendar_csv_is_io_error() {
        let dir = ghost();
        let path = dir.path().join("calendar.csv");
        fs::create_dir(&path).unwrap();
        assert!(matches!(load(dir.path()), Err(LoaderError::Io(p, _)) if p == path));
    }

    #[test]
    fn names_differing_only_in_case_are_duplicate() {
        let dir = ghost();
        fs::write(dir.path().join("calendar.csv"), "").unwrap();
        fs::write(dir.path().join("CALENDAR.csv"), "").unwrap();
        if fs::read_dir(dir.path()).unwrap().count() < 2 {
            return; // case-insensitive file system: only one file exists
        }
        let err = load(dir.path()).unwrap_err();
        assert!(matches!(&err, LoaderError::CalendarDuplicate { paths } if paths.len() == 2));
        let msg = err.to_string();
        assert!(
            msg.contains("calendar.csv") && msg.contains("CALENDAR.csv"),
            "{msg}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_is_not_counted() {
        let dir = ghost();
        let real = dir.path().join("real.csv");
        fs::write(&real, "0101,x\n").unwrap();
        std::os::unix::fs::symlink(&real, dir.path().join("calendar.csv")).unwrap();
        assert_eq!(load(dir.path()).unwrap(), CalendarTable::default());
    }
}
