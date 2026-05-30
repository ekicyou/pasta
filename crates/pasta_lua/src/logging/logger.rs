//! PastaLogger - Instance-specific file logger with rotation.
//!
//! Each ghost instance can have its own log file with automatic rotation.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use crate::loader::LoggingConfig;

/// PastaLogger - Instance-specific file logger with rotation.
///
/// Each instance manages its own log file with non-blocking writes
/// and automatic rotation.
pub struct PastaLogger {
    /// Absolute path to the log file.
    log_path: PathBuf,

    /// Non-blocking writer.
    writer: Mutex<NonBlocking>,

    /// Worker guard - must be kept alive to ensure logs are flushed.
    /// Dropped when PastaLogger is dropped.
    _guard: WorkerGuard,
}

impl PastaLogger {
    /// Create a new logger with the given configuration.
    ///
    /// # Arguments
    /// * `base_dir` - Base directory (load_dir)
    /// * `config` - Logging configuration (or None for defaults)
    ///
    /// # Returns
    /// * `Ok(Self)` - Logger created successfully
    /// * `Err(e)` - Failed to create log directory or file
    pub fn new(base_dir: &Path, config: Option<&LoggingConfig>) -> io::Result<Self> {
        let config = config.cloned().unwrap_or_default();

        // Compute absolute log path
        let log_path = base_dir.join(&config.file_path);

        // Validate path is within profile directory
        Self::validate_path(base_dir, &log_path)?;

        // Create log directory if needed
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Get directory and filename for the appender
        let log_dir = log_path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid log path"))?;
        let log_file_name = log_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid log filename"))?;

        // Create file appender with no rotation (fixed filename)
        let appender = RollingFileAppender::builder()
            .rotation(Rotation::NEVER)
            .filename_prefix(log_file_name)
            .build(log_dir)
            .map_err(io::Error::other)?;

        // Create non-blocking writer
        let (writer, guard) = tracing_appender::non_blocking(appender);

        Ok(Self {
            log_path,
            writer: Mutex::new(writer),
            _guard: guard,
        })
    }

    /// Validate that the log path is within allowed directories.
    ///
    /// Prevents path traversal attacks by ensuring the log path
    /// stays within the profile/pasta/ directory.
    fn validate_path(base_dir: &Path, log_path: &Path) -> io::Result<()> {
        // The log path should be within profile/ directory
        let relative = log_path.strip_prefix(base_dir).map_err(|_| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Log path must be relative to base_dir: {:?}", log_path),
            )
        })?;

        let relative_str = relative.to_string_lossy();
        if !relative_str.starts_with("profile") {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Log path must be within profile/ directory: {:?}", log_path),
            ));
        }

        // Check for path traversal attempts
        if relative_str.contains("..") {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Path traversal not allowed in log path",
            ));
        }

        Ok(())
    }

    /// Get the absolute path to the log file.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Write bytes to the log file.
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let mut writer = self
            .writer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        writer.write(buf)
    }

    /// Flush the log buffer.
    pub fn flush(&self) -> io::Result<()> {
        let mut writer = self
            .writer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        writer.flush()
    }

    /// Check if logging is enabled.
    pub fn is_enabled(&self) -> bool {
        true
    }
}

impl Drop for PastaLogger {
    fn drop(&mut self) {
        // Flush remaining logs before dropping
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        // Create profile directory structure
        std::fs::create_dir_all(base_dir.join("profile/pasta/logs")).unwrap();

        let config = LoggingConfig::default();
        let logger = PastaLogger::new(base_dir, Some(&config)).unwrap();

        assert!(logger.is_enabled());
        assert!(logger.log_path().to_string_lossy().contains("pasta.log"));
    }

    #[test]
    fn test_path_validation_rejects_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        let config = LoggingConfig {
            file_path: "../outside.log".to_string(),
            rotation_days: 7,
            level: "debug".to_string(),
            filter: None,
        };

        let result = PastaLogger::new(base_dir, Some(&config));
        assert!(result.is_err());
    }

    #[test]
    fn test_path_validation_rejects_non_profile() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        let config = LoggingConfig {
            file_path: "other/logs/pasta.log".to_string(),
            rotation_days: 7,
            level: "debug".to_string(),
            filter: None,
        };

        let result = PastaLogger::new(base_dir, Some(&config));
        assert!(result.is_err());
    }

    #[test]
    fn test_log_file_is_fixed_name_no_date_suffix() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        std::fs::create_dir_all(base_dir.join("profile/pasta/logs")).unwrap();

        let logger = PastaLogger::new(base_dir, None).unwrap();

        // Write something to trigger file creation
        logger.write(b"test log line\n").unwrap();
        logger.flush().unwrap();

        // Check that the log file is exactly "pasta.log" with no date suffix
        let log_dir = base_dir.join("profile/pasta/logs");
        let entries: Vec<_> = std::fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert!(
            entries.contains(&"pasta.log".to_string()),
            "Log file should be exactly 'pasta.log', found: {:?}",
            entries
        );
        // No date-suffixed files should exist
        let date_files: Vec<_> = entries
            .iter()
            .filter(|n| n.starts_with("pasta.log.") && n.len() > "pasta.log".len())
            .collect();
        assert!(
            date_files.is_empty(),
            "No date-suffixed log files should exist, found: {:?}",
            date_files
        );
    }
}
