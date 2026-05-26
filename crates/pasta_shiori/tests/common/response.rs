//! SHIORI/3.0 response parser for test assertions.

use std::collections::HashMap;
use std::fmt;

/// Parsed SHIORI/3.0 response.
#[derive(Debug, Clone)]
pub struct ShioriResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub value: Option<String>,
}

/// Error type for SHIORI response parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ShioriResponseError {
    Empty,
    MissingStatusLine,
    InvalidStatusLine(String),
    InvalidHeaderLine(String),
}

impl fmt::Display for ShioriResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty response"),
            Self::MissingStatusLine => write!(f, "missing status line"),
            Self::InvalidStatusLine(line) => write!(f, "invalid status line: {}", line),
            Self::InvalidHeaderLine(line) => write!(f, "invalid header line: {}", line),
        }
    }
}

impl std::error::Error for ShioriResponseError {}

impl ShioriResponse {
    /// Parse a SHIORI/3.0 response string into structured fields.
    pub fn parse(text: &str) -> Result<Self, ShioriResponseError> {
        if text.is_empty() {
            return Err(ShioriResponseError::Empty);
        }

        let mut lines = text.split("\r\n");

        // Parse status line: "SHIORI/3.0 200 OK"
        let status_line = lines.next().ok_or(ShioriResponseError::MissingStatusLine)?;
        if status_line.is_empty() {
            return Err(ShioriResponseError::MissingStatusLine);
        }

        let (status_code, status_text) = parse_status_line(status_line)?;

        // Parse headers
        let mut headers = HashMap::new();
        let mut value = None;

        for line in lines {
            if line.is_empty() {
                break; // End of headers (empty line = \r\n\r\n)
            }
            let (key, val) = parse_header_line(line)?;

            // Extract Value header specially
            if key.eq_ignore_ascii_case("Value") {
                value = Some(val.to_string());
            } else {
                headers.insert(key.to_string(), val.to_string());
            }
        }

        Ok(ShioriResponse {
            status_code,
            status_text,
            headers,
            value,
        })
    }

    /// Get a header value by name (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Check if the response is a success (2xx status code).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }
}

fn parse_status_line(line: &str) -> Result<(u16, String), ShioriResponseError> {
    // Expected: "SHIORI/3.0 200 OK"
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return Err(ShioriResponseError::InvalidStatusLine(line.to_string()));
    }
    let code: u16 = parts[1]
        .parse()
        .map_err(|_| ShioriResponseError::InvalidStatusLine(line.to_string()))?;
    let text = parts[2].to_string();
    Ok((code, text))
}

fn parse_header_line(line: &str) -> Result<(&str, &str), ShioriResponseError> {
    let pos = line
        .find(": ")
        .ok_or_else(|| ShioriResponseError::InvalidHeaderLine(line.to_string()))?;
    Ok((&line[..pos], &line[pos + 2..]))
}
