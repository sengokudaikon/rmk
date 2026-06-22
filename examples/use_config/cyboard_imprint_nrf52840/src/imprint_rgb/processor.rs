use embassy_nrf::pwm::SequencePwm;
use rmk::event::{
    ConnectionStatusChangeEvent, KeyboardEvent, KeyboardEventPos, LayerChangeEvent,
    LedIndicatorEvent,
};
use rmk::macros::processor;

use crate::imprint_rgb::{
    BRIGHTNESS_CAP, CHAIN_LENGTH, LAYER_COLORS, LEFT_LED_KEY_MAP, RIGHT_LED_KEY_MAP,
};

use super::frame::LedFrame;
use super::writer::PwmWriter;

const PRESS_TICKS: u8 = 4;
const MAX_ACTIVE_PRESSES: usize = 8;

#[derive(Clone, Copy)]
struct ActivePress {
    led_index: u8,
    ticks_remaining: u8,
}

#[processor(
    subscribe = [
        ConnectionStatusChangeEvent,
        KeyboardEvent,
        LayerChangeEvent,
        LedIndicatorEvent,
    ],
    poll_interval = 250
)]
pub struct ImprintRgb {
    frame: LedFrame,
    writer: PwmWriter,
    is_left: bool,
    initialized: bool,
    current_layer: u8,
    caps_lock: bool,
    active_presses: [ActivePress; MAX_ACTIVE_PRESSES],
    active_press_count: usize,
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
            active_presses: [ActivePress {
                led_index: 0,
                ticks_remaining: 0,
            }; MAX_ACTIVE_PRESSES],
            active_press_count: 0,
        }
    }

    async fn on_connection_status_change_event(&mut self, _event: ConnectionStatusChangeEvent) {}

    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if !event.pressed {
            return;
        }
        if let KeyboardEventPos::Key(pos) = event.pos {
            if let Some(led) = self.find_led(pos.row, pos.col) {
                self.activate_press(led);
                self.render();
            }
        }
    }

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

        let presses_expired = self.decay_active_presses();
        if presses_expired {
            self.render();
        }

        if !self.frame.is_dirty() {
            return;
        }

        self.writer.encode(&self.frame);
        self.writer.play().await;
        self.frame.mark_clean();
    }

    fn find_led(&self, row: u8, col: u8) -> Option<usize> {
        let map = if self.is_left {
            &LEFT_LED_KEY_MAP[..CHAIN_LENGTH]
        } else {
            &RIGHT_LED_KEY_MAP[..CHAIN_LENGTH]
        };
        map.iter().position(|&(r, c)| r == row && c == col)
    }

    fn activate_press(&mut self, led_index: usize) {
        for i in 0..self.active_press_count {
            if self.active_presses[i].led_index == led_index as u8 {
                self.active_presses[i].ticks_remaining = PRESS_TICKS;
                return;
            }
        }
        if self.active_press_count < MAX_ACTIVE_PRESSES {
            self.active_presses[self.active_press_count] = ActivePress {
                led_index: led_index as u8,
                ticks_remaining: PRESS_TICKS,
            };
            self.active_press_count += 1;
        }
    }

    fn decay_active_presses(&mut self) -> bool {
        let mut changed = false;
        let mut i = 0;
        while i < self.active_press_count {
            self.active_presses[i].ticks_remaining -= 1;
            if self.active_presses[i].ticks_remaining == 0 {
                self.active_press_count -= 1;
                self.active_presses[i] = self.active_presses[self.active_press_count];
                changed = true;
            } else {
                i += 1;
            }
        }
        changed
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

        for i in 0..self.active_press_count {
            let led = self.active_presses[i].led_index as usize;
            self.frame.set_led(led, BRIGHTNESS_CAP, BRIGHTNESS_CAP, BRIGHTNESS_CAP);
        }
    }
}
