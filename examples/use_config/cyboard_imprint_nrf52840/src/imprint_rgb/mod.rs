mod frame;
mod writer;
pub(crate) mod processor;

pub(crate) use processor::ImprintRgb;

include!(concat!(env!("OUT_DIR"), "/led_map_constants.rs"));

pub(crate) const LED_COUNT: usize = CHAIN_LENGTH;
pub(crate) const SEQ_LEN: usize = LED_COUNT * 24 + 1;

pub(crate) const T1H: u16 = 0x8000 | 13;
pub(crate) const T0H: u16 = 0x8000 | 7;
pub(crate) const RES: u16 = 0x8000;

pub(crate) const BRIGHTNESS_CAP: u8 = 32;
