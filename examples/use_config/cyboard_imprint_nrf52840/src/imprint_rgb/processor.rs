use embassy_nrf::pwm::SequencePwm;
use rmk::event::{ConnectionStatusChangeEvent, KeyboardEvent, KeyboardEventPos};
use rmk::macros::processor;

use crate::imprint_rgb::{LED_COUNT, MAPPING_MODE};

use super::frame::LedFrame;
use super::writer::PwmWriter;

#[processor(
    subscribe = [ConnectionStatusChangeEvent, KeyboardEvent],
    poll_interval = 250
)]
pub struct ImprintRgb {
    frame: LedFrame,
    writer: PwmWriter,
    wave_done: bool,
    map_index: usize,
    map_initialized: bool,
}

impl ImprintRgb {
    pub fn new(pwm: SequencePwm<'static>) -> Self {
        Self {
            frame: LedFrame::new(),
            writer: PwmWriter::new(pwm),
            wave_done: false,
            map_index: 0,
            map_initialized: false,
        }
    }

    async fn on_connection_status_change_event(&mut self, _event: ConnectionStatusChangeEvent) {}

    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if !MAPPING_MODE {
            return;
        }
        if event.pressed {
            if let KeyboardEventPos::Key(_) = event.pos {
                self.advance_map();
            }
        }
    }

    async fn poll(&mut self) {
        if MAPPING_MODE {
            self.poll_mapping().await;
            return;
        }

        if !self.wave_done {
            self.run_wave().await;
            self.wave_done = true;
        }

        if !self.frame.is_dirty() {
            return;
        }

        self.writer.encode(&self.frame);
        self.writer.play().await;
        self.frame.mark_clean();
    }

    async fn poll_mapping(&mut self) {
        if !self.map_initialized {
            self.advance_map();
            self.map_initialized = true;
        }

        if !self.frame.is_dirty() {
            return;
        }

        self.writer.encode(&self.frame);
        self.writer.play().await;
        self.frame.mark_clean();
    }

    fn advance_map(&mut self) {
        self.frame.clear();
        self.frame.set_led(self.map_index, 0, 3, 12);
        self.map_index = (self.map_index + 1) % LED_COUNT;
    }

    async fn run_wave(&mut self) {
        self.frame.clear();

        for led in 0..LED_COUNT {
            self.frame.set_led(led, 0, 2, 8);
            self.writer.encode(&self.frame);
            self.writer.play().await;
        }

        self.frame.set_all(0, 1, 3);
    }
}
