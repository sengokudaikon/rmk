use embassy_nrf::pwm::SequencePwm;
use rmk::event::ConnectionStatusChangeEvent;
use rmk::macros::processor;
use crate::imprint_rgb::LED_COUNT;
use super::frame::LedFrame;
use super::writer::PwmWriter;
#[processor(
    subscribe = [ConnectionStatusChangeEvent],
    poll_interval = 250
)]
pub struct ImprintRgb {
    frame: LedFrame,
    writer: PwmWriter,
    wave_done: bool,
}

impl ImprintRgb {
    pub fn new(pwm: SequencePwm<'static>) -> Self {
        Self {
            frame: LedFrame::new(),
            writer: PwmWriter::new(pwm),
            wave_done: false,
        }
    }

    async fn on_connection_status_change_event(&mut self, _event: ConnectionStatusChangeEvent) {}

    async fn poll(&mut self) {
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
