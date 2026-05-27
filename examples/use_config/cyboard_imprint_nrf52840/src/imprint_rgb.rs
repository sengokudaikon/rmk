use embassy_nrf::pwm::{SequenceConfig, SequencePwm, SingleSequenceMode, SingleSequencer};
use embassy_time::Timer;
use rmk::event::ConnectionStatusChangeEvent;
use rmk::macros::processor;

const LED_COUNT: usize = 64;
const SEQ_LEN: usize = LED_COUNT * 24 + 1;

const T1H: u16 = 0x8000 | 13;
const T0H: u16 = 0x8000 | 7;
const RES: u16 = 0x8000;

#[processor(
    subscribe = [ConnectionStatusChangeEvent],
    poll_interval = 250
)]
pub struct ImprintRgb {
    pwm: SequencePwm<'static>,
    seq_words: [u16; SEQ_LEN],
    seq_config: SequenceConfig,
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
            wave_done: false,
        }
    }

    async fn on_connection_status_change_event(&mut self, _event: ConnectionStatusChangeEvent) {}

    async fn poll(&mut self) {
        if !self.wave_done {
            for i in 0..SEQ_LEN - 1 {
                self.seq_words[i] = T0H;
            }

            for led in 0..LED_COUNT {
                self.encode_led(led, (0, 2, 8));
                let pwm = &mut self.pwm;
                let words: &[u16] = &self.seq_words;
                let seq = SingleSequencer::new(pwm, words, self.seq_config.clone());
                let _ = seq.start(SingleSequenceMode::Times(1));
                Timer::after_millis(15).await;
            }

            self.wave_done = true;
        }

        for led in 0..LED_COUNT {
            self.encode_led(led, (0, 1, 3));
        }

        let pwm = &mut self.pwm;
        let words: &[u16] = &self.seq_words;
        let seq = SingleSequencer::new(pwm, words, self.seq_config.clone());
        let _ = seq.start(SingleSequenceMode::Times(1));
        Timer::after_millis(50).await;
    }

    fn encode_led(&mut self, led: usize, (red, green, blue): (u8, u8, u8)) {
        let offset = led * 24;
        let mut out = offset;
        for byte in [green, red, blue] {
            for bit in (0..8).rev() {
                self.seq_words[out] = if byte & (1 << bit) != 0 { T1H } else { T0H };
                out += 1;
            }
        }
    }
}
