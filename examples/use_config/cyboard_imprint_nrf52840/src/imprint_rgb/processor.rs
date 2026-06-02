use embassy_nrf::pwm::SequencePwm;
use rmk::event::{
    ConnectionStatusChangeEvent, LayerChangeEvent, LedIndicatorEvent,
};
use rmk::macros::processor;

use crate::imprint_rgb::{CHAIN_LENGTH, LAYER_COLORS, LEFT_LED_KEY_MAP, RIGHT_LED_KEY_MAP};

use super::frame::LedFrame;
use super::writer::PwmWriter;

#[processor(
    subscribe = [ConnectionStatusChangeEvent, LayerChangeEvent, LedIndicatorEvent],
    poll_interval = 250
)]
pub struct ImprintRgb {
    frame: LedFrame,
    writer: PwmWriter,
    is_left: bool,
    initialized: bool,
    current_layer: u8,
    caps_lock: bool,
}

impl ImprintRgb {
    pub fn new(pwm: SequencePwm<'static>, is_left: bool) -> Self {
        Self {
            frame: LedFrame::new(),
            writer: PwmWriter::new(pwm),
            is_left,
            initialized: false,
            current_layer: 0,
            caps_lock: false,
        }
    }

    async fn on_connection_status_change_event(&mut self, _event: ConnectionStatusChangeEvent) {}

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        self.current_layer = event.0;
        self.render();
    }

    async fn on_led_indicator_event(&mut self, event: LedIndicatorEvent) {
        let new_caps = event.0.caps_lock();
        if self.caps_lock != new_caps {
            self.caps_lock = new_caps;
            self.render();
        }
    }

    async fn poll(&mut self) {
        if !self.initialized {
            self.render();
            self.initialized = true;
        }

        if !self.frame.is_dirty() {
            return;
        }

        self.writer.encode(&self.frame);
        self.writer.play().await;
        self.frame.mark_clean();
    }

    fn render(&mut self) {
        self.frame.clear();

        let map = if self.is_left {
            &LEFT_LED_KEY_MAP[..CHAIN_LENGTH]
        } else {
            &RIGHT_LED_KEY_MAP[..CHAIN_LENGTH]
        };

        let color_idx = if self.caps_lock {
            3
        } else {
            self.current_layer.min((LAYER_COLORS.len() - 1) as u8) as usize
        };
        let [r, g, b] = LAYER_COLORS[color_idx];

        for (led, &(row, col)) in map.iter().enumerate() {
            if row == 255 && col == 255 {
                continue;
            }
            self.frame.set_led(led, r, g, b);
        }
    }
}
