use embassy_nrf::pwm::{SequenceConfig, SequencePwm, SingleSequenceMode, SingleSequencer};
use embassy_time::Timer;

const LED_COUNT: usize = 64;
const SEQ_LEN: usize = LED_COUNT * 24 + 1;

const T1H: u16 = 0x8000 | 13;
const T0H: u16 = 0x8000 | 7;
const RES: u16 = 0x8000;

// Placeholder subscription required by #[processor] macro.
// Event-driven RGB (connection/layer/LED) is a future feature.
use rmk::event::ConnectionStatusChangeEvent;
use rmk::macros::processor;

#[processor(
    subscribe = [ConnectionStatusChangeEvent],
    poll_interval = 250
)]
pub struct ImprintRgb {
    pwm: SequencePwm<'static>,
    seq_words: [u16; SEQ_LEN],
    seq_config: SequenceConfig,
    frame: [u32; LED_COUNT],
    dirty: bool,
    wave_done: bool,
}

impl ImprintRgb {
    pub fn new(pwm: SequencePwm<'static>) -> Self {
        let mut seq_words = [0; SEQ_LEN];
        seq_words[SEQ_LEN - 1] = RES;
        let mut seq_config = SequenceConfig::default();
        seq_config.end_delay = 799;
        Self {
            pwm,
            seq_words,
            seq_config,
            frame: [0; LED_COUNT],
            dirty: true,
            wave_done: false,
        }
    }

    async fn on_connection_status_change_event(&mut self, _event: ConnectionStatusChangeEvent) {}

    async fn poll(&mut self) {
        if !self.wave_done {
            self.run_wave().await;
            self.wave_done = true;
        }

        if !self.dirty {
            return;
        }

        self.encode_frame();

        let pwm = &mut self.pwm;
        let words: &[u16] = &self.seq_words;
        let seq = SingleSequencer::new(pwm, words, self.seq_config.clone());
        let _ = seq.start(SingleSequenceMode::Times(1));
        Timer::after_millis(50).await;

        self.dirty = false;
    }

    #[allow(dead_code)]
    fn set_led(&mut self, index: usize, r: u8, g: u8, b: u8) {
        let color = u32::from_le_bytes([g, r, b, 0]);
        if self.frame[index] != color {
            self.frame[index] = color;
            self.dirty = true;
        }
    }

    fn set_all(&mut self, r: u8, g: u8, b: u8) {
        let color = u32::from_le_bytes([g, r, b, 0]);
        for i in 0..LED_COUNT {
            if self.frame[i] != color {
                self.frame[i] = color;
                self.dirty = true;
            }
        }
    }

    fn encode_frame(&mut self) {
        for (led, &color) in self.frame.iter().enumerate() {
            let offset = led * 24;
            let mut out = offset;
            let bytes = color.to_le_bytes();
            for &byte in &bytes[0..3] {
                for bit in (0..8).rev() {
                    self.seq_words[out] = if byte & (1 << bit) != 0 { T1H } else { T0H };
                    out += 1;
                }
            }
        }
    }

    async fn run_wave(&mut self) {
        for i in 0..SEQ_LEN - 1 {
            self.seq_words[i] = T0H;
        }

        for led in 0..LED_COUNT {
            self.frame[led] = u32::from_le_bytes([2, 0, 8, 0]);

            let offset = led * 24;
            let mut out = offset;
            for &byte in &[2u8, 0, 8] {
                for bit in (0..8).rev() {
                    self.seq_words[out] = if byte & (1 << bit) != 0 { T1H } else { T0H };
                    out += 1;
                }
            }

            let pwm = &mut self.pwm;
            let words: &[u16] = &self.seq_words;
            let seq = SingleSequencer::new(pwm, words, self.seq_config.clone());
            let _ = seq.start(SingleSequenceMode::Times(1));
            Timer::after_millis(15).await;
        }

        self.set_all(0, 1, 3);
    }
}
