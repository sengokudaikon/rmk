use embassy_nrf::spim::Spim;
use rmk::event::{ConnectionStatusChangeEvent, LayerChangeEvent, LedIndicatorEvent};
use rmk::macros::processor;
use rmk::types::ble::BleState;

const LED_COUNT: usize = 64;
const RESET_BYTES: usize = 80;
const FRAME_BYTES: usize = LED_COUNT * 24 + RESET_BYTES;

const ZERO_FRAME: u8 = 0x40;
const ONE_FRAME: u8 = 0x70;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RgbMode {
    Alive,
    Advertising,
    Connected,
}

#[processor(
    subscribe = [
        ConnectionStatusChangeEvent,
        LayerChangeEvent,
        LedIndicatorEvent
    ],
    poll_interval = 250
)]
pub struct ImprintRgb {
    spim: Spim<'static>,
    frame: [u8; FRAME_BYTES],
    tick: u8,
    active_layer: u8,
    caps_lock: bool,
    mode: RgbMode,
    dirty: bool,
}

impl ImprintRgb {
    pub fn new(spim: Spim<'static>) -> Self {
        Self {
            spim,
            frame: [0; FRAME_BYTES],
            tick: 0,
            active_layer: 0,
            caps_lock: false,
            mode: RgbMode::Alive,
            dirty: true,
        }
    }

    async fn on_connection_status_change_event(&mut self, event: ConnectionStatusChangeEvent) {
        self.mode = match event.0.ble.state {
            BleState::Advertising => RgbMode::Advertising,
            BleState::Connected => RgbMode::Connected,
            BleState::Inactive => RgbMode::Alive,
        };
        self.dirty = true;
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        self.active_layer = event.0;
        self.dirty = true;
    }

    async fn on_led_indicator_event(&mut self, event: LedIndicatorEvent) {
        self.caps_lock = event.0.caps_lock();
        self.dirty = true;
    }

    async fn poll(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        if self.mode == RgbMode::Advertising {
            self.dirty = true;
        }

        if !self.dirty {
            return;
        }

        self.render();
        let _ = self.spim.write_from_ram(&self.frame).await;
        self.dirty = false;
    }

    fn render(&mut self) {
        let color = if self.caps_lock {
            (8, 0, 0)
        } else {
            match (self.mode, self.active_layer) {
                (RgbMode::Advertising, _) if self.tick % 2 == 0 => (0, 0, 0),
                (RgbMode::Advertising, _) => (0, 0, 6),
                (_, 0) => (0, 3, 5),
                (_, 1) => (0, 7, 0),
                _ => (6, 0, 6),
            }
        };

        for led in 0..LED_COUNT {
            self.encode_led(led, color);
        }

        for byte in &mut self.frame[LED_COUNT * 24..] {
            *byte = 0;
        }
    }

    fn encode_led(&mut self, led: usize, (red, green, blue): (u8, u8, u8)) {
        let offset = led * 24;
        let mut out = offset;

        for byte in [green, red, blue] {
            for bit in (0..8).rev() {
                self.frame[out] = if byte & (1 << bit) != 0 { ONE_FRAME } else { ZERO_FRAME };
                out += 1;
            }
        }
    }
}
