use crate::{bus::BusResult, devices::IoDevice};
use crate::bus::BusError;

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

    pub fn mouse_button(&mut self, button: u32, down: bool) {
        if (1..4).contains(&button) {
            let bit = 1u32 << (27 - button);
            if down { self.mouse |= bit; } else { self.mouse &= !bit; }
        }
    }

    pub fn keyboard_input(&mut self, bytes: &[u8]) -> BusResult<()>{
        if self.key_cnt + bytes.len() > self.key_buf.len() {
            return Err(BusError::Device("keyboard buffer full".into()));
        }
        let dst = &mut self.key_buf[self.key_cnt .. self.key_cnt + bytes.len()];
        dst.copy_from_slice(bytes);
        self.key_cnt += bytes.len();
        Ok(())
    }

    fn read_mouse_and_kb_status(&self) -> u32 {
        let mut m = self.mouse;
        if self.key_cnt > 0 {
            m |= 0x1000_0000; // keyboard ready
        }
        m
    }

    pub(crate) fn read_keyboard_data(&mut self) -> u32 {
        if self.key_cnt == 0 {
            return 0;
        }
        let sc = self.key_buf[0];
        // shift buffer left
        for i in 1..self.key_cnt {
            self.key_buf[i - 1] = self.key_buf[i];
        }
        self.key_cnt -= 1;
        sc as u32
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
