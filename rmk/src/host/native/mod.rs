//! RMK-native host protocol runtime (Milestone 1: read-only).
//!
//! This module implements the firmware-side host service for the RMK native
//! protocol. It runs over the existing 32-byte host HID channel shared with the
//! Vial path, so no new USB descriptors or BLE characteristics are required for
//! the first milestone.
//!
//! The temporary transport framing is isolated in the [`transport`] submodule
//! and documented there.

use heapless::Vec;
use postcard_rpc::{Endpoint, Key};
use rmk_types::protocol::rmk::{
    DeviceCapabilities, GetCapabilities, GetCurrentLayer, GetKeyAction, GetMatrixState, GetVersion, KeyPosition,
    MatrixState, ProtocolVersion,
};

#[cfg(feature = "bulk_transfer")]
use rmk_types::protocol::rmk::{GetKeymapBulk, GetKeymapBulkRequest, GetKeymapBulkResponse};

#[cfg(feature = "_ble")]
use crate::channel::HOST_BLE_REPLY;
use crate::channel::HOST_REQUEST_CHANNEL;
#[cfg(not(feature = "_no_usb"))]
use crate::channel::HOST_USB_REPLY;
use crate::config::RmkConfig;
use crate::core_traits::Runnable;
use crate::host::context::KeyboardContext;
use crate::{
    COMBO_MAX_LENGTH, COMBO_MAX_NUM, FORK_MAX_NUM, MACRO_DATA_SIZE, MACRO_SPACE_SIZE, MAX_PATTERNS_PER_KEY,
    MORSE_MAX_NUM, NUM_BLE_PROFILE, SPLIT_PERIPHERALS_NUM,
};

#[cfg(feature = "bulk_transfer")]
use crate::BULK_SIZE;

pub mod transport;

/// Largest request/response body we serialize before chunking.
const MAX_MESSAGE_SIZE: usize = 256;

/// Byte buffer used for a single request or response payload.
///
/// NOTE: the capacity is written as a literal because `heapless::Vec`'s type
/// alias normalizes differently with named constants in some const-generic
/// positions. Keep this in sync with [`MAX_MESSAGE_SIZE`].
type MessageBuf = heapless::Vec<u8, 256>;

/// Host-side service for the RMK native protocol.
///
/// Implements the read-only endpoints required for osyn parity with the
/// Vial-based app. Construct it from a [`KeyboardContext`] and pass it to the
/// task runner, just like [`VialService`](super::via::VialService).
pub struct HostService<'a> {
    ctx: &'a KeyboardContext<'a>,
}

impl<'a> HostService<'a> {
    /// Create a new native-protocol host service.
    ///
    /// `config` is accepted for API compatibility with [`VialService`] but is
    /// not used by the read-only milestone 1 implementation.
    pub fn new(ctx: &'a KeyboardContext<'a>, _config: &RmkConfig<'static>) -> Self {
        Self { ctx }
    }

    /// Dispatch a parsed request to the appropriate endpoint handler.
    async fn handle_request(&self, req: transport::Request) -> Option<(Key, MessageBuf)> {
        if req.key == GetVersion::REQ_KEY.to_bytes() {
            return Some((GetVersion::RESP_KEY, serialize(&ProtocolVersion::CURRENT)));
        }

        if req.key == GetCapabilities::REQ_KEY.to_bytes() {
            return Some((GetCapabilities::RESP_KEY, serialize(&self.device_capabilities())));
        }

        if req.key == GetCurrentLayer::REQ_KEY.to_bytes() {
            return Some((GetCurrentLayer::RESP_KEY, serialize(&self.ctx.active_layer())));
        }

        if req.key == GetMatrixState::REQ_KEY.to_bytes() {
            return Some((GetMatrixState::RESP_KEY, serialize(&self.matrix_state())));
        }

        if req.key == GetKeyAction::REQ_KEY.to_bytes() {
            let action = match postcard::from_bytes::<KeyPosition>(&req.body) {
                Ok(pos) => self.ctx.get_action(pos.layer, pos.row, pos.col),
                Err(_) => {
                    warn!("RMK protocol: invalid KeyPosition body");
                    rmk_types::action::KeyAction::No
                }
            };
            return Some((GetKeyAction::RESP_KEY, serialize(&action)));
        }

        #[cfg(feature = "bulk_transfer")]
        if req.key == GetKeymapBulk::REQ_KEY.to_bytes() {
            return Some((GetKeymapBulk::RESP_KEY, self.bulk_keymap_response(&req.body)));
        }

        warn!("RMK protocol: unknown endpoint key");
        None
    }

    /// Build the capabilities descriptor from compile-time constants and the
    /// live keymap configuration.
    fn device_capabilities(&self) -> DeviceCapabilities {
        let (rows, cols, layers) = self.ctx.keymap_dimensions();

        #[cfg(feature = "bulk_transfer")]
        let max_bulk_keys = BULK_SIZE as u8;
        #[cfg(not(feature = "bulk_transfer"))]
        let max_bulk_keys = 0;

        DeviceCapabilities {
            num_layers: layers as u8,
            num_rows: rows as u8,
            num_cols: cols as u8,
            num_encoders: self.ctx.num_encoders() as u8,
            max_combos: COMBO_MAX_NUM as u8,
            max_combo_keys: COMBO_MAX_LENGTH as u8,
            max_macros: 32,
            macro_space_size: MACRO_SPACE_SIZE as u16,
            max_morse: MORSE_MAX_NUM as u8,
            max_patterns_per_key: MAX_PATTERNS_PER_KEY as u8,
            max_forks: FORK_MAX_NUM as u8,
            storage_enabled: cfg!(feature = "storage"),
            lighting_enabled: false,
            is_split: cfg!(feature = "split"),
            num_split_peripherals: SPLIT_PERIPHERALS_NUM as u8,
            ble_enabled: cfg!(feature = "_ble"),
            num_ble_profiles: NUM_BLE_PROFILE as u8,
            max_payload_size: transport::REPORT_SIZE as u16,
            max_bulk_keys,
            macro_chunk_size: MACRO_DATA_SIZE as u16,
            bulk_transfer_supported: cfg!(feature = "bulk_transfer"),
        }
    }

    /// Convert the live matrix state into the protocol [`MatrixState`] bitmap.
    fn matrix_state(&self) -> MatrixState {
        let (rows, cols, _) = self.ctx.keymap_dimensions();
        let row_len = cols.div_ceil(8);
        let meaningful = rows * row_len;

        // The internal MatrixState::read_all() writes each row's bytes in
        // reverse order (Vial compatibility). Read into a scratch buffer and
        // un-reverse per row so the native protocol gets natural row-major
        // ordering with bit 0 = col 0.
        let mut raw = [0u8; 30];
        self.ctx.read_matrix_state(&mut raw);

        let mut pressed_bitmap = Vec::new();
        for r in 0..rows {
            let row_start = r * row_len;
            let row_end = row_start + row_len;
            for i in 0..row_len {
                // read_all emitted row_bytes.iter().rev()
                pressed_bitmap.push(raw[row_end - 1 - i]).ok();
            }
        }

        // If the board is smaller than MATRIX_BITMAP_SIZE, only report the
        // meaningful bytes. Hosts decode using num_rows/num_cols from caps.
        pressed_bitmap.truncate(meaningful);

        MatrixState { pressed_bitmap }
    }

    /// Handle `keymap/bulk_get` when `bulk_transfer` is enabled.
    #[cfg(feature = "bulk_transfer")]
    fn bulk_keymap_response(&self, body: &[u8]) -> MessageBuf {
        let req = match postcard::from_bytes::<GetKeymapBulkRequest>(body) {
            Ok(r) => r,
            Err(_) => {
                warn!("RMK protocol: invalid GetKeymapBulkRequest body");
                return serialize(&GetKeymapBulkResponse { actions: Vec::new() });
            }
        };

        let (rows, cols, _) = self.ctx.keymap_dimensions();
        let start_row = req.start_row as usize;
        let start_col = req.start_col as usize;

        let mut actions: Vec<rmk_types::action::KeyAction, BULK_SIZE> = Vec::new();
        for i in 0..req.count as usize {
            let mut col = start_col + i;
            let row = start_row + col / cols;
            col %= cols;
            let action = if row < rows {
                self.ctx.get_action(req.layer, row as u8, col as u8)
            } else {
                rmk_types::action::KeyAction::No
            };
            if actions.push(action).is_err() {
                break;
            }
        }

        serialize(&GetKeymapBulkResponse { actions })
    }
}

impl Runnable for HostService<'_> {
    async fn run(&mut self) -> ! {
        loop {
            let (transport, output_data) = HOST_REQUEST_CHANNEL.receive().await;
            match transport::parse_request(&output_data) {
                Ok(req) => {
                    let txn_id = req.txn_id;
                    if let Some((resp_key, body)) = self.handle_request(req).await {
                        let frames = transport::encode_response(resp_key, &body, txn_id);
                        for frame in frames {
                            send_host_reply(transport, frame).await;
                        }
                    }
                }
                Err(e) => {
                    warn!("RMK protocol request parse error: {:?}", e);
                }
            }
        }
    }
}

/// Send a 32-byte host reply back to the transport's reply channel, awaiting
/// capacity so multi-frame responses are not dropped.
async fn send_host_reply(transport: rmk_types::connection::ConnectionType, _reply: [u8; 32]) {
    match transport {
        #[cfg(not(feature = "_no_usb"))]
        rmk_types::connection::ConnectionType::Usb => HOST_USB_REPLY.send(_reply).await,
        #[cfg(feature = "_ble")]
        rmk_types::connection::ConnectionType::Ble => HOST_BLE_REPLY.send(_reply).await,
        #[allow(unreachable_patterns)]
        _ => {}
    }
}

/// Serialize a response body into a heapless byte vector.
fn serialize<T: serde::Serialize>(val: &T) -> MessageBuf {
    let mut buf = [0u8; MAX_MESSAGE_SIZE];
    match postcard::to_slice(val, &mut buf) {
        Ok(bytes) => {
            let mut v = MessageBuf::new();
            if v.extend_from_slice(bytes).is_err() {
                warn!("RMK protocol response exceeded MAX_MESSAGE_SIZE");
            }
            v
        }
        Err(e) => {
            warn!("RMK protocol serialization error: {:?}", e);
            MessageBuf::new()
        }
    }
}
