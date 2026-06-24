//! Temporary raw-HID transport framing for the RMK native protocol.
//!
//! This module is intentionally self-contained so it can be replaced with
//! postcard-rpc/COBS over USB bulk or BLE serial later without touching the
//! endpoint handlers.
//!
//! Each request/response is a sequence of one or more 32-byte HID reports:
//!
//! ```text
//! byte 0: flags
//!   bit 7: continuation (1 = more frames follow in this message)
//!   bit 6: reserved (must be 0)
//!   bits 0-5: frame sequence number (0-based)
//! byte 1: transaction id (host sets, firmware echoes)
//! byte 2: body length (postcard body bytes; only meaningful in first frame)
//! byte 3-31: payload (up to 29 bytes per frame)
//! ```
//!
//! Reassembled payload format:
//!
//! ```text
//! bytes 0-7:   endpoint key (REQ_KEY for requests, RESP_KEY for responses)
//! bytes 8..:   postcard-serialized request/response body (body length bytes)
//! ```
//!
//! Milestone 1 requests are small enough to fit in a single frame. Responses may
//! be split across multiple frames (e.g. `status/matrix/get` and
//! `keymap/bulk_get`).

use postcard_rpc::Key;

use super::MessageBuf;

pub const REPORT_SIZE: usize = 32;
const FRAME_HEADER_SIZE: usize = 3;
pub const FRAME_PAYLOAD_SIZE: usize = REPORT_SIZE - FRAME_HEADER_SIZE; // 29

/// Largest message we will serialize before chunking.
pub const MAX_MESSAGE_SIZE: usize = super::MAX_MESSAGE_SIZE;
const MAX_RESPONSE_FRAMES: usize = MAX_MESSAGE_SIZE.div_ceil(FRAME_PAYLOAD_SIZE) + 1;

/// Parsed request from the host.
#[derive(Debug, PartialEq)]
pub struct Request {
    /// 8-byte endpoint request key (compared against `Endpoint::REQ_KEY`).
    pub key: [u8; 8],
    pub body: MessageBuf,
    pub txn_id: u8,
}

/// Parse a single-frame request report.
///
/// Milestone 1 only supports single-frame requests; all current request
/// bodies fit comfortably within 29 payload bytes.
pub fn parse_request(report: &[u8; REPORT_SIZE]) -> Result<Request, ParseError> {
    let flags = report[0];
    if flags & 0x80 != 0 {
        return Err(ParseError::MultiFrameRequest);
    }
    if flags & 0x40 != 0 {
        return Err(ParseError::ReservedFlag);
    }
    if flags & 0x3F != 0 {
        return Err(ParseError::UnexpectedSequence);
    }

    let txn_id = report[1];
    let body_len = report[2] as usize;
    let payload = &report[FRAME_HEADER_SIZE..];

    if payload.len() < 8 {
        return Err(ParseError::TooShort);
    }

    let key: [u8; 8] = payload[..8].try_into().map_err(|_| ParseError::TooShort)?;
    let mut body: MessageBuf = heapless::Vec::new();
    let body_end = 8 + body_len;
    if body_end > payload.len() {
        return Err(ParseError::TooShort);
    }
    body.extend_from_slice(&payload[8..body_end])
        .map_err(|_| ParseError::TooShort)?;

    Ok(Request { key, body, txn_id })
}

/// Encode a response body into one or more 32-byte HID reports.
pub fn encode_response(
    resp_key: Key,
    body: &[u8],
    txn_id: u8,
) -> heapless::Vec<[u8; REPORT_SIZE], MAX_RESPONSE_FRAMES> {
    let mut message: MessageBuf = heapless::Vec::new();
    // Ignoring Result::Err here: both pushes are guaranteed to fit because
    // MAX_MESSAGE_SIZE is larger than any response we generate.
    let _ = message.extend_from_slice(&resp_key.to_bytes());
    let _ = message.extend_from_slice(body);

    let mut frames: heapless::Vec<[u8; REPORT_SIZE], MAX_RESPONSE_FRAMES> = heapless::Vec::new();
    let mut seq = 0u8;
    let mut offset = 0;

    while offset < message.len() {
        let mut frame = [0u8; REPORT_SIZE];
        let remaining = message.len() - offset;
        let chunk_len = remaining.min(FRAME_PAYLOAD_SIZE);
        let is_last = remaining <= FRAME_PAYLOAD_SIZE;

        frame[0] = if is_last { seq } else { seq | 0x80 };
        frame[1] = txn_id;
        if offset == 0 {
            // Body length is only placed in the first frame header; it
            // does not appear in the reassembled message.
            frame[2] = body.len() as u8;
        }
        frame[FRAME_HEADER_SIZE..FRAME_HEADER_SIZE + chunk_len].copy_from_slice(&message[offset..offset + chunk_len]);

        frames.push(frame).ok();
        offset += chunk_len;
        seq = seq.wrapping_add(1) & 0x3F;
    }

    frames
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseError {
    MultiFrameRequest,
    ReservedFlag,
    UnexpectedSequence,
    TooShort,
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcard_rpc::Endpoint;
    use rmk_types::protocol::rmk::GetVersion;

    #[test]
    fn round_trip_single_frame_request() {
        let mut report = [0u8; REPORT_SIZE];
        report[0] = 0; // seq 0, no continuation
        report[1] = 0xAB; // txn id
        report[2] = 0; // body length 0
        let key = GetVersion::REQ_KEY.to_bytes();
        report[FRAME_HEADER_SIZE..FRAME_HEADER_SIZE + 8].copy_from_slice(&key);

        let req = parse_request(&report).unwrap();
        assert_eq!(req.key, key);
        assert_eq!(req.txn_id, 0xAB);
        assert!(req.body.is_empty());
    }

    #[test]
    fn rejects_multi_frame_request() {
        let mut report = [0u8; REPORT_SIZE];
        report[0] = 0x80; // continuation set
        report[1..FRAME_HEADER_SIZE + 8].copy_from_slice(&[0u8; 10]);
        assert_eq!(parse_request(&report), Err(ParseError::MultiFrameRequest));
    }

    #[test]
    fn rejects_reserved_flag() {
        let mut report = [0u8; REPORT_SIZE];
        report[0] = 0x40; // reserved flag
        report[1..FRAME_HEADER_SIZE + 8].copy_from_slice(&[0u8; 10]);
        assert_eq!(parse_request(&report), Err(ParseError::ReservedFlag));
    }

    #[test]
    fn encode_response_chunks_and_preserves_key() {
        let body: MessageBuf = {
            let mut v = heapless::Vec::new();
            // 35 bytes of body so the response needs two frames.
            for i in 0..35u8 {
                v.push(i).unwrap();
            }
            v
        };
        let frames = encode_response(GetVersion::RESP_KEY, &body, 0xCD);
        assert_eq!(frames.len(), 2);

        // First frame: continuation set, txn id echoed, body length set,
        // begins with key.
        assert_eq!(frames[0][0] & 0x80, 0x80);
        assert_eq!(frames[0][1], 0xCD);
        assert_eq!(frames[0][2], body.len() as u8);
        assert_eq!(
            &frames[0][FRAME_HEADER_SIZE..FRAME_HEADER_SIZE + 8],
            &GetVersion::RESP_KEY.to_bytes()
        );

        // Last frame: continuation clear, txn id echoed.
        assert_eq!(frames[1][0] & 0x80, 0);
        assert_eq!(frames[1][1], 0xCD);

        // Reassemble payload by taking every frame's full payload except
        // trimming the last frame's trailing zeros.
        let mut payload = MessageBuf::new();
        for (i, frame) in frames.iter().enumerate() {
            let is_last = i == frames.len() - 1;
            let chunk = &frame[FRAME_HEADER_SIZE..];
            if is_last {
                // Last frame's real payload length = total remaining bytes.
                let already = payload.len();
                let total = 8 + body.len();
                let remaining = total - already;
                payload.extend_from_slice(&chunk[..remaining]).unwrap();
            } else {
                payload.extend_from_slice(chunk).unwrap();
            }
        }
        assert_eq!(&payload[..8], &GetVersion::RESP_KEY.to_bytes());
        assert_eq!(&payload[8..], &body[..]);
    }
}
