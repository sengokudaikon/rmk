pub(crate) const LED_COUNT: usize = 64;
pub(crate) const SEQ_LEN: usize = LED_COUNT * 24 + 1;

pub(crate) const T1H: u16 = 0x8000 | 13;
pub(crate) const T0H: u16 = 0x8000 | 7;
pub(crate) const RES: u16 = 0x8000;

pub(crate) const BRIGHTNESS_CAP: u8 = 32;

pub(crate) const MAPPING_MODE: bool = false;

mod frame;
mod writer;
pub(crate) mod processor;

pub(crate) use processor::ImprintRgb;
