use crate::{bus::BusResult, devices::IoDevice};

#[derive(Debug, Default)]
pub struct Input {
    mouse: u32,
    key_buf: [u8; 16],
    key_cnt: usize,
}

impl Input {
    pub fn mouse_moved(&mut self, mouse_x: i32, mouse_y: i32) {
        if (0..4096).contains(&mouse_x) {
            self.mouse = (self.mouse & !0x0000_0FFF) | (mouse_x as u32);
        }
        if (0..4096).contains(&mouse_y) {
            self.mouse = (self.mouse & !0x00FF_F000) | ((mouse_y as u32) << 12);
        }
    }

    pub fn mouse_button(&mut self, button: i32, down: bool) {
        if (1..4).contains(&button) {
            let bit = 1u32 << (27 - (button as u32));
            if down { self.mouse |= bit; } else { self.mouse &= !bit; }
        }
    }

    pub fn keyboard_input(&mut self, scancodes: &[u8]) {
        let free = self.key_buf.len().saturating_sub(self.key_cnt);
        if scancodes.len() <= free {
            self.key_buf[self.key_cnt..self.key_cnt + scancodes.len()].copy_from_slice(scancodes);
            self.key_cnt += scancodes.len();
        }
    }

    fn read_mouse_and_kb_status(&self) -> u32 {
        let mut v = self.mouse;
        if self.key_cnt > 0 {
            v |= 0x1000_0000;
        }
        v
    }

    fn read_keyboard_data(&mut self) -> u32 {
        if self.key_cnt > 0 {
            let sc = self.key_buf[0];
            self.key_cnt -= 1;
            self.key_buf.copy_within(1..1 + self.key_cnt, 0);
            sc as u32
        } else {
            0
        }
    }
}

impl IoDevice for Input {
    fn read(&mut self, offset: u32) -> BusResult<u32> {
        match offset {
            24 => Ok(self.read_mouse_and_kb_status()),
            28 => Ok(self.read_keyboard_data()),
            _ => Ok(0),
        }
    }

    fn write(&mut self, _offset: u32, _value: u32) -> BusResult<()> {
        Ok(())
    }
}
