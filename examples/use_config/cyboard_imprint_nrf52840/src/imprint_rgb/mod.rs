const LED_COUNT: usize = 64;
const SEQ_LEN: usize = LED_COUNT * 24 + 1;

const T1H: u16 = 0x8000 | 13;
const T0H: u16 = 0x8000 | 7;
const RES: u16 = 0x8000;

const BRIGHTNESS_CAP: u8 = 32;

mod frame;

mod writer;
mod processor;