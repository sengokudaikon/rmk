use crate::imprint_rgb::{BRIGHTNESS_CAP, LED_COUNT};

struct LedFrame {
    data: [u32; LED_COUNT],
    dirty: bool,
}

impl LedFrame {
    pub(crate) fn new() -> Self {
        Self {
            data: [0; LED_COUNT],
            dirty: false,
        }
    }

    pub(crate) fn set_led(&mut self, index: usize, r: u8, g: u8, b: u8) {
        let color = pack(r, g, b);
        if self.data[index] != color {
            self.data[index] = color;
            self.dirty = true;
        }
    }

    pub(crate) fn set_all(&mut self, r: u8, g: u8, b: u8) {
        let color = pack(r, g, b);
        for i in 0..LED_COUNT {
            if self.data[i] != color {
                self.data[i] = color;
                self.dirty = true;
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.set_all(0, 0, 0);
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

fn pack(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([
        g.min(BRIGHTNESS_CAP),
        r.min(BRIGHTNESS_CAP),
        b.min(BRIGHTNESS_CAP),
        0,
    ])
}