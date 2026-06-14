//! `transport` モジュールの **フレームコーデック**（`Content-Length` フレーミング・
//! バイト長・ヘッダ堅牢性・厳密長/各種エラー）インラインテスト外出し（task 2.4・C1）。
//!
//! 移動のみ（振る舞い不変）。元の単一 `mod tests`（~740行）を凝集境界で 2 兄弟へ分割した
//! うちのコーデック単体クラスタ。純粋・Lua非依存・`Cursor` 上で完結する `write_frame` /
//! `read_frame` の単体仕様を集約する。`use super::*;` で本番項目へ到達（本番可視性は不変）。

use super::*;

use std::io::Cursor;

use serde_json::json;

// -----------------------------------------------------------------------
// Frame codec unit tests (byte-length framing, header robustness, exactness)
// -----------------------------------------------------------------------

/// `write_frame` then `read_frame` round-trips an arbitrary JSON value.
#[test]
fn frame_round_trip_ascii() {
    let value = json!({
        "seq": 1,
        "type": "request",
        "command": "initialize",
        "arguments": { "adapterID": "pasta" }
    });

    let mut buf: Vec<u8> = Vec::new();
    write_frame(&mut buf, &value).expect("write_frame must succeed");

    let mut reader = Cursor::new(buf);
    let read = read_frame(&mut reader)
        .expect("read_frame must succeed")
        .expect("a frame must be present");
    assert_eq!(read, value, "round-trip must preserve the JSON value");
}

/// The `Content-Length` header MUST be the UTF-8 BYTE length, not the char
/// count. A multi-byte payload (Japanese) proves byte-vs-char correctness.
#[test]
fn content_length_is_byte_length_not_char_count() {
    // "こんにちは" is 5 chars but 15 UTF-8 bytes.
    let payload = "こんにちは";
    assert_eq!(payload.chars().count(), 5);
    assert_eq!(payload.len(), 15);

    let value = json!({ "text": payload });

    let mut buf: Vec<u8> = Vec::new();
    write_frame(&mut buf, &value).expect("write must succeed");

    // The emitted header must carry the BYTE length of the JSON body.
    let text = String::from_utf8(buf.clone()).expect("frame is UTF-8");
    let body = serde_json::to_vec(&value).unwrap();
    let expected_header = format!("Content-Length: {}\r\n\r\n", body.len());
    assert!(
        text.starts_with(&expected_header),
        "header must report the BYTE length ({}), got frame starting: {:?}",
        body.len(),
        &text[..expected_header.len().min(text.len())]
    );

    // And it must round-trip exactly (no over/under-read of the multi-byte body).
    let mut reader = Cursor::new(buf);
    let read = read_frame(&mut reader)
        .expect("read must succeed")
        .expect("a frame must be present");
    assert_eq!(read, value, "multi-byte body must round-trip intact");
    assert_eq!(read["text"], json!(payload));
}

/// `read_frame` is robust to extra and reordered headers; only
/// `Content-Length` is significant, matched case-insensitively.
#[test]
fn read_frame_tolerates_extra_and_reordered_headers() {
    let body = br#"{"ok":true}"#;
    // Extra header BEFORE Content-Length, a different-cased name, and a
    // trailing extra header — all must be ignored except Content-Length.
    let mut frame: Vec<u8> = Vec::new();
    frame.extend_from_slice(b"X-Extra: hello\r\n");
    frame.extend_from_slice(format!("content-length: {}\r\n", body.len()).as_bytes());
    frame.extend_from_slice(b"X-Another: world\r\n");
    frame.extend_from_slice(b"\r\n");
    frame.extend_from_slice(body);

    let mut reader = Cursor::new(frame);
    let read = read_frame(&mut reader)
        .expect("read must succeed with extra/reordered headers")
        .expect("a frame must be present");
    assert_eq!(read, json!({ "ok": true }));
}

/// `read_frame` reads EXACTLY N body bytes and does not consume the start of
/// a following frame (no over-read).
#[test]
fn read_frame_reads_exactly_n_bytes_and_leaves_the_next_frame() {
    let first = json!({ "a": 1 });
    let second = json!({ "b": 2 });

    let mut buf: Vec<u8> = Vec::new();
    write_frame(&mut buf, &first).unwrap();
    write_frame(&mut buf, &second).unwrap();

    let mut reader = Cursor::new(buf);
    let r1 = read_frame(&mut reader).unwrap().expect("first frame");
    assert_eq!(r1, first, "first frame parsed");
    let r2 = read_frame(&mut reader).unwrap().expect("second frame");
    assert_eq!(r2, second, "second frame intact (no over-read of the first)");
    // A third read hits clean EOF between frames.
    assert!(
        read_frame(&mut reader).unwrap().is_none(),
        "clean EOF between frames yields Ok(None)"
    );
}

/// A missing `Content-Length` header is a framing error (not silent).
#[test]
fn read_frame_missing_content_length_is_error() {
    let mut frame: Vec<u8> = Vec::new();
    frame.extend_from_slice(b"X-Only: nope\r\n\r\n");
    frame.extend_from_slice(br#"{"x":1}"#);
    let mut reader = Cursor::new(frame);
    let err = read_frame(&mut reader).expect_err("missing Content-Length must error");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

/// A truncated body (fewer than `Content-Length` bytes) is an error, not a
/// short/partial parse.
#[test]
fn read_frame_truncated_body_is_error() {
    let mut frame: Vec<u8> = Vec::new();
    // Claim 20 bytes but provide far fewer.
    frame.extend_from_slice(b"Content-Length: 20\r\n\r\n");
    frame.extend_from_slice(br#"{"x":1}"#);
    let mut reader = Cursor::new(frame);
    let err = read_frame(&mut reader).expect_err("truncated body must error");
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
}

/// A non-numeric `Content-Length` value is a framing error (`InvalidData`),
/// not a silent skip — a malformed client cannot smuggle an unframed body.
#[test]
fn read_frame_non_numeric_content_length_is_error() {
    let mut frame: Vec<u8> = Vec::new();
    frame.extend_from_slice(b"Content-Length: abc\r\n\r\n");
    frame.extend_from_slice(br#"{"x":1}"#);
    let mut reader = Cursor::new(frame);
    let err = read_frame(&mut reader).expect_err("non-numeric Content-Length must error");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(
        err.to_string().contains("invalid Content-Length"),
        "error should name the bad header, got: {err}"
    );
}

/// EOF in the MIDDLE of a header block (some header bytes seen, no blank
/// line yet) is `UnexpectedEof` — distinct from the clean `Ok(None)` EOF
/// BETWEEN frames. This is the truncated-header half of the EOF contract.
#[test]
fn read_frame_eof_mid_header_is_unexpected_eof() {
    // A header line was read, but the stream ends before the blank line.
    let frame: Vec<u8> = b"Content-Length: 7\r\n".to_vec();
    let mut reader = Cursor::new(frame);
    let err = read_frame(&mut reader).expect_err("EOF mid-header must error");
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    assert!(
        err.to_string().contains("header"),
        "error should mention the header block, got: {err}"
    );
}

/// A body that is not valid UTF-8 is `InvalidData` (the DAP body is JSON,
/// which is UTF-8 by definition), not a panic or a lossy decode.
#[test]
fn read_frame_non_utf8_body_is_error() {
    let body: &[u8] = &[0xFF, 0xFE, 0xFD, 0xFC]; // invalid UTF-8
    let mut frame: Vec<u8> = Vec::new();
    frame.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
    frame.extend_from_slice(body);
    let mut reader = Cursor::new(frame);
    let err = read_frame(&mut reader).expect_err("non-UTF-8 body must error");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

/// A body that is valid UTF-8 but not valid JSON is `InvalidData`. Also
/// pins `Content-Length: 0` (an empty body is NOT valid JSON, so a zero
/// length frame is rejected rather than yielding a phantom value).
#[test]
fn read_frame_invalid_json_body_is_error() {
    // Valid UTF-8, invalid JSON.
    let body = b"not json at all";
    let mut frame: Vec<u8> = Vec::new();
    frame.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
    frame.extend_from_slice(body);
    let mut reader = Cursor::new(frame);
    let err = read_frame(&mut reader).expect_err("invalid JSON body must error");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);

    // Content-Length: 0 → empty body → invalid JSON → InvalidData.
    let mut zero: Vec<u8> = b"Content-Length: 0\r\n\r\n".to_vec();
    // Append a following frame to prove the zero-length read consumed nothing.
    write_frame(&mut zero, &json!({"after": true})).unwrap();
    let mut reader = Cursor::new(zero);
    let err = read_frame(&mut reader).expect_err("zero-length body must error");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

/// G3 hardening regression: an attacker-controlled oversized
/// `Content-Length` is rejected as `InvalidData` BEFORE the body buffer is
/// allocated, so a malicious client cannot drive an unbounded allocation
/// (memory-exhaustion DoS) with a single cheap header. At the limit itself
/// the frame is still read normally (the guard is exclusive-above), which
/// here surfaces as `UnexpectedEof` because no body bytes follow.
#[test]
fn read_frame_rejects_oversized_content_length_before_allocating() {
    // One byte over the cap → rejected up front (InvalidData, names the cap).
    let mut over: Vec<u8> = Vec::new();
    over.extend_from_slice(
        format!("Content-Length: {}\r\n\r\n", MAX_CONTENT_LENGTH + 1).as_bytes(),
    );
    let mut reader = Cursor::new(over);
    let err = read_frame(&mut reader).expect_err("oversized Content-Length must error");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(
        err.to_string().contains("exceeds"),
        "error should name the exceeded cap, got: {err}"
    );

    // Exactly AT the cap the guard does not fire: the read proceeds and
    // fails only because the body is absent (UnexpectedEof, not InvalidData).
    let mut at: Vec<u8> = Vec::new();
    at.extend_from_slice(format!("Content-Length: {MAX_CONTENT_LENGTH}\r\n\r\n").as_bytes());
    let mut reader = Cursor::new(at);
    let err = read_frame(&mut reader).expect_err("missing body must error");
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof, "at-limit passes the guard");
}

/// Header lines terminated by bare `\n` (no `\r`) are tolerated: the
/// terminator trim accepts both `\r\n` and `\n`, so a lenient client still
/// frames correctly.
#[test]
fn read_frame_tolerates_bare_lf_header_endings() {
    let body = br#"{"lf":true}"#;
    let mut frame: Vec<u8> = Vec::new();
    frame.extend_from_slice(format!("Content-Length: {}\n", body.len()).as_bytes());
    frame.extend_from_slice(b"\n");
    frame.extend_from_slice(body);
    let mut reader = Cursor::new(frame);
    let read = read_frame(&mut reader)
        .expect("bare-LF headers must parse")
        .expect("a frame must be present");
    assert_eq!(read, json!({ "lf": true }));
}
