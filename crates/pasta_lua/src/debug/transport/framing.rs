//! Frame codec: DAP-compliant `Content-Length` framing — pure, Lua-free,
//! unit-testable.
//!
//! This submodule owns the byte/JSON wire codec of the [`Transport`]: the
//! `Content-Length: <N>\r\n\r\n<json>` frame format where `N` is the **byte**
//! length of the UTF-8 JSON body (NOT its char count — multi-byte UTF-8 such as
//! Japanese makes the two differ). [`write_frame`] serializes a value into a
//! frame; [`read_frame`] parses one frame back, robust to header ordering and
//! extra headers, with a DoS guard on the attacker-controlled length.
//!
//! I/O only — never touches Lua.
//!
//! [`Transport`]: super::Transport

use std::io::{self, BufRead, Write};

use serde_json::Value;

/// The DAP header that carries the body byte length.
const CONTENT_LENGTH: &str = "Content-Length";

/// Upper bound for an inbound frame body accepted by [`read_frame`].
///
/// The `Content-Length` value is **attacker-controlled** (the TCP debugger
/// client is a trust boundary): without a cap, a single malicious header could
/// drive an arbitrarily large body allocation before any byte of the body is
/// read (memory-exhaustion DoS). Real DAP messages are tiny; 16 MiB is far
/// above any legitimate frame while keeping the worst-case allocation bounded.
pub(crate) const MAX_CONTENT_LENGTH: usize = 16 * 1024 * 1024;

/// Serialize `value` into a `Content-Length`-framed DAP wire frame and write it
/// to `out`.
///
/// The body is compact UTF-8 JSON; the header reports its **byte** length
/// (`buf.len()` of the UTF-8 encoding, NOT the char count), then a blank
/// `\r\n\r\n` separates the header block from the body. The whole frame is
/// flushed so the peer can read it immediately.
///
/// I/O only — never touches Lua.
pub(crate) fn write_frame<W: Write>(out: &mut W, value: &Value) -> io::Result<()> {
    // Compact JSON body. `to_vec` yields the exact UTF-8 bytes; the header MUST
    // use this byte length (multi-byte UTF-8 makes bytes != chars).
    let body = serde_json::to_vec(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(out, "{CONTENT_LENGTH}: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()
}

/// Read one `Content-Length`-framed DAP wire frame from `reader` and parse the
/// body into a [`serde_json::Value`].
///
/// Parsing is robust to header ordering and to extra headers: the header block
/// is read line by line until a blank line (the `\r\n\r\n` separator), and only
/// the `Content-Length` header is significant (its name is matched
/// case-insensitively, surrounding whitespace trimmed). Then EXACTLY that many
/// body bytes are read (no over- or under-read), decoded as UTF-8, and parsed.
///
/// Returns `Ok(None)` on a clean EOF *before* any header bytes (the peer closed
/// the connection between frames). Any malformed frame (missing
/// `Content-Length`, truncated body, non-UTF-8, invalid JSON) is an
/// [`io::Error`].
///
/// I/O only — never touches Lua.
pub(crate) fn read_frame<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    let mut saw_any_header_byte = false;

    // (1) Read the header block, line by line, until a blank line.
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            // EOF. If it landed exactly between frames (no header bytes read),
            // it's a clean close; otherwise the frame was truncated.
            if saw_any_header_byte {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF in the middle of a frame header block",
                ));
            }
            return Ok(None);
        }
        saw_any_header_byte = true;

        // The blank line (`\r\n` or `\n`) terminates the header block.
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        // Parse `Header-Name: value`; only Content-Length matters. Robust to
        // ordering and to additional headers (which are ignored).
        if let Some((name, value)) = trimmed.split_once(':')
            && name.trim().eq_ignore_ascii_case(CONTENT_LENGTH)
        {
            let parsed = value.trim().parse::<usize>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid Content-Length value: {value:?}"),
                )
            })?;
            content_length = Some(parsed);
        }
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "frame header block missing Content-Length",
        )
    })?;

    // DoS guard: the length is attacker-controlled, so reject absurd values
    // BEFORE allocating the body buffer (see [`MAX_CONTENT_LENGTH`]).
    if len > MAX_CONTENT_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Content-Length {len} exceeds the maximum {MAX_CONTENT_LENGTH}"),
        ));
    }

    // (2) Read EXACTLY `len` body bytes (no over/under-read).
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;

    // (3) Decode UTF-8 and parse JSON.
    let text = String::from_utf8(body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let value = serde_json::from_str::<Value>(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(value))
}
