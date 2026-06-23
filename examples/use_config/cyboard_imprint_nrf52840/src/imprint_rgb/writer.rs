use embassy_nrf::pwm::{SequenceConfig, SequencePwm, SingleSequenceMode, SingleSequencer};
use embassy_time::Timer;

use crate::imprint_rgb::{RES, SEQ_LEN, T0H, T1H};
use super::frame::LedFrame;

pub(crate) struct PwmWriter {
    pwm: SequencePwm<'static>,
    words: [u16; SEQ_LEN],
    config: SequenceConfig,
}

impl PwmWriter {
    pub(crate) fn new(pwm: SequencePwm<'static>) -> Self {
        let mut words = [0; SEQ_LEN];
        words[SEQ_LEN - 1] = RES;
        let mut config = SequenceConfig::default();
        config.end_delay = 799;
        Self { pwm, words, config }
    }

    pub(crate) fn encode(&mut self, frame: &LedFrame) {
        for (led, &color) in frame.data.iter().enumerate() {
            let offset = led * 24;
            let mut out = offset;
            let bytes = color.to_le_bytes();
            for &byte in &bytes[0..3] {
                for bit in (0..8).rev() {
                    self.words[out] = if byte & (1 << bit) != 0 { T1H } else { T0H };
                    out += 1;
                }
            }
        }
    }

    pub(crate) async fn play(&mut self) {
        let pwm = &mut self.pwm;
        let words: &[u16] = &self.words;
        let seq = SingleSequencer::new(pwm, words, self.config.clone());
        let _ = seq.start(SingleSequenceMode::Times(1));
        Timer::after_millis(5).await;
    }
}
