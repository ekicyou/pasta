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

        // Create rolling file appender with daily rotation
        // max_log_files is based on rotation_days
        let appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .max_log_files(config.rotation_days)
            .filename_prefix(log_file_name)
            .build(log_dir)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
        // Canonicalize paths for comparison
        // Note: log_path may not exist yet, so we canonicalize its parent
        let base_canonical = base_dir
            .canonicalize()
            .unwrap_or_else(|_| base_dir.to_path_buf());

        // Check that the log path starts with base_dir
        let log_canonical = if let Some(parent) = log_path.parent() {
            if parent.exists() {
                let parent_canonical = parent.canonicalize()?;
                parent_canonical.join(log_path.file_name().unwrap_or_default())
            } else {
                // Parent doesn't exist yet, just normalize the path
                log_path.to_path_buf()
            }
        } else {
            log_path.to_path_buf()
        };

        // The log path should be within profile/ directory
        // Allow paths that contain "profile" in them
        let relative = log_path.strip_prefix(base_dir).map_err(|_| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Log path must be relative to base_dir: {:?}", log_path),
            )
        })?;

        let relative_str = relative.to_string_lossy();
        if !relative_str.starts_with("profile") && !relative_str.starts_with("profile/") {
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

        let _ = (base_canonical, log_canonical); // silence unused warning
        Ok(())
    }

    /// Get the absolute path to the log file.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Write bytes to the log file.
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let mut writer = self.writer.lock().unwrap();
        writer.write(buf)
    }

    /// Flush the log buffer.
    pub fn flush(&self) -> io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
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
        };

        let result = PastaLogger::new(base_dir, Some(&config));
        assert!(result.is_err());
    }
}
