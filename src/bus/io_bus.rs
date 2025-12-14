use crate::{
    bus::{BusError, BusResult},
    devices::IoDevice,
};
use crate::devices::spi::SpiDevice;

#[derive(Debug)]
pub struct IoBus {
    io_start: u32,

    pub timer: Box<dyn IoDevice>,
    pub switches: Box<dyn IoDevice>,
    pub serial: Option<Box<dyn IoDevice>>,
    pub spi: [Option<Box<dyn SpiDevice>>; 4],
    pub input: Box<dyn IoDevice>,
    pub clipboard: Option<Box<dyn IoDevice>>,
    pub leds: Option<Box<dyn IoDevice>>,

    pub spi_selected: u32,
}

impl IoBus {
    pub fn new(
        io_start: u32,
        timer: Box<dyn IoDevice>,
        switches: Box<dyn IoDevice>,
        input: Box<dyn IoDevice>,
    ) -> Self {
        Self {
            io_start,
            timer,
            switches,
            serial: None,
            spi: [None, None, None, None],
            input,
            clipboard: None,
            leds: None,
            spi_selected: 0,
        }
    }

    #[inline]
    fn progress_dec(progress: &mut u32) {
        *progress = progress.saturating_sub(1);
    }

    /// Progress-aware read:
    /// - offset 0 (ms counter) => progress--
    /// - offset 24 (mouse/kbd status) => progress-- hvis kbd *ikke* ready
    pub fn read_word_with_progress(&mut self, addr: u32, progress: &mut u32) -> BusResult<u32> {
        if addr < self.io_start {
            return Err(BusError::Unmapped(addr));
        }
        let off = addr - self.io_start;

        match off {
            0 => {
                Self::progress_dec(progress);
                self.timer.read(0)
            }
            4 => self.switches.read(4),

            8 => self.serial.as_deref_mut().map(|d| d.read(8)).unwrap_or(Ok(0)),
            12 => self.serial.as_deref_mut().map(|d| d.read(12)).unwrap_or(Ok(0)),

            16 => {
                let idx = (self.spi_selected & 3) as usize;
                self.spi[idx].as_deref_mut()
                    .map(|d| d.read_data())
                    .unwrap_or(Ok(255))
            }
            20 => Ok(1),

            24 => {
                let v = self.input.read(24)?;
                if (v & 0x1000_0000) == 0 {
                    Self::progress_dec(progress);
                }
                Ok(v)
            }
            28 => self.input.read(28),

            40 => self.clipboard.as_deref_mut().map(|d| d.read(40)).unwrap_or(Ok(0)),
            44 => self.clipboard.as_deref_mut().map(|d| d.read(44)).unwrap_or(Ok(0)),

            _ => Ok(0),
        }
    }

    pub fn write_word(&mut self, addr: u32, value: u32) -> BusResult<()> {
        if addr < self.io_start {
            return Err(BusError::Unmapped(addr));
        }
        let off = addr - self.io_start;

        match off {
            4 => self.leds.as_deref_mut().map(|d| d.write(4, value)).unwrap_or(Ok(())),
            8 => self.serial.as_deref_mut().map(|d| d.write(8, value)).unwrap_or(Ok(())),

            16 => {
                let idx = (self.spi_selected & 3) as usize;
                self.spi[idx].as_deref_mut()
                    .map(|d| d.write_data(value))
                    .unwrap_or(Ok(()))
            }
            20 => {
                self.spi_selected = value & 3;
                Ok(())
            }

            40 => self.clipboard.as_deref_mut().map(|d| d.write(40, value)).unwrap_or(Ok(())),
            44 => self.clipboard.as_deref_mut().map(|d| d.write(44, value)).unwrap_or(Ok(())),

            _ => Ok(()),
        }
    }
}

