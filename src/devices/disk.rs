use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::bus::{BusError, BusResult};
use crate::devices::spi::SpiDevice;

#[derive(Debug, Clone, Copy)]
enum DiskState {
    DiskCommand,
    DiskRead,
    DiskWrite,
    DiskWriting,
}

#[derive(Debug)]
pub struct Disk {
    state: DiskState,
    file: Option<File>,
    offset: u32,

    rx_buf: [u32; 128],
    rx_idx: usize,

    tx_buf: [u32; 128 + 2],
    tx_cnt: usize,
    tx_idx: i32, // C brugte -1 som “første read giver tx_buf[0]”
}

impl Disk {
    pub fn new(filename: Option<&str>) -> BusResult<Self> {
        let mut disk = Self {
            state: DiskState::DiskCommand,
            file: None,
            offset: 0,
            rx_buf: [0; 128],
            rx_idx: 0,
            tx_buf: [0; 130],
            tx_cnt: 0,
            tx_idx: 0,
        };

        if let Some(name) = filename {
            let mut f = File::options()
                .read(true)
                .write(true)
                .open(name)
                .map_err(|e| BusError::Device(format!("Can't open file \"{name}\": {e}")))?;

            // Check for filesystem-only image, starting directly at sector 1 (DiskAdr 29)
            let mut tmp = [0u32; 128];
            read_sector(Some(&mut f), &mut tmp)?;
            disk.offset = if tmp[0] == 0x9B1E_A38D { 0x80002 } else { 0 };

            // rewind ikke strengt nødvendigt; seek sker per command
            disk.file = Some(f);
        }

        Ok(disk)
    }

    fn run_command(&mut self) -> BusResult<()> {
        let cmd = self.rx_buf[0];

        let arg = (self.rx_buf[1] << 24)
            | (self.rx_buf[2] << 16)
            | (self.rx_buf[3] << 8)
            | (self.rx_buf[4] << 0);

        match cmd {
            81 => {
                // READ
                self.state = DiskState::DiskRead;
                self.tx_buf[0] = 0;
                self.tx_buf[1] = 254;

                let sec = arg.wrapping_sub(self.offset);
                seek_sector(self.file.as_mut(), sec)?;
                read_sector(self.file.as_mut(), (&mut self.tx_buf[2..2 + 128]).try_into().unwrap())?;

                self.tx_cnt = 2 + 128;
            }
            88 => {
                // WRITE
                self.state = DiskState::DiskWrite;
                let sec = arg.wrapping_sub(self.offset);
                seek_sector(self.file.as_mut(), sec)?;
                self.tx_buf[0] = 0;
                self.tx_cnt = 1;
            }
            _ => {
                // default ack
                self.tx_buf[0] = 0;
                self.tx_cnt = 1;
            }
        }

        self.tx_idx = -1;
        Ok(())
    }
}

impl SpiDevice for Disk {
    fn write_data(&mut self, value: u32) -> BusResult<()> {
        self.tx_idx += 1;

        match self.state {
            DiskState::DiskCommand => {
                // Ignorer 0xFF “idle bytes” medmindre vi allerede er i gang med en kommando
                if (value as u8) != 0xFF || self.rx_idx != 0 {
                    self.rx_buf[self.rx_idx] = value;
                    self.rx_idx += 1;
                    if self.rx_idx == 6 {
                        self.run_command()?;
                        self.rx_idx = 0;
                    }
                }
            }

            DiskState::DiskRead => {
                if self.tx_idx >= self.tx_cnt as i32 {
                    self.state = DiskState::DiskCommand;
                    self.tx_cnt = 0;
                    self.tx_idx = 0;
                }
            }

            DiskState::DiskWrite => {
                // vent på data token 0xFE
                if value == 254 {
                    self.state = DiskState::DiskWriting;
                }
            }

            DiskState::DiskWriting => {
                if self.rx_idx < 128 {
                    self.rx_buf[self.rx_idx] = value;
                }
                self.rx_idx += 1;

                if self.rx_idx == 128 {
                    // skriv 512 bytes (128 words)
                    write_sector(self.file.as_mut(), &self.rx_buf)?;
                }

                if self.rx_idx == 130 {
                    // done: send status byte 5 (som C)
                    self.tx_buf[0] = 5;
                    self.tx_cnt = 1;
                    self.tx_idx = -1;
                    self.rx_idx = 0;
                    self.state = DiskState::DiskCommand;
                }
            }
        }

        Ok(())
    }

    fn read_data(&mut self) -> BusResult<u32> {
        if self.tx_idx >= 0 && (self.tx_idx as usize) < self.tx_cnt {
            Ok(self.tx_buf[self.tx_idx as usize])
        } else {
            Ok(255)
        }
    }
}

// --- helpers (1:1 med C) ---

fn seek_sector(file: Option<&mut File>, secnum: u32) -> BusResult<()> {
    if let Some(f) = file {
        f.seek(SeekFrom::Start(secnum as u64 * 512))
            .map_err(|e| BusError::Device(format!("seek failed: {e}")))?;
    }
    Ok(())
}

fn read_sector(file: Option<&mut File>, buf: &mut [u32; 128]) -> BusResult<()> {
    let mut bytes = [0u8; 512];
    if let Some(f) = file {
        f.read_exact(&mut bytes)
            .map_err(|e| BusError::Device(format!("read failed: {e}")))?;
    }
    for i in 0..128 {
        buf[i] = (bytes[i * 4 + 0] as u32)
            | ((bytes[i * 4 + 1] as u32) << 8)
            | ((bytes[i * 4 + 2] as u32) << 16)
            | ((bytes[i * 4 + 3] as u32) << 24);
    }
    Ok(())
}

fn write_sector(file: Option<&mut File>, buf: &[u32; 128]) -> BusResult<()> {
    if let Some(f) = file {
        let mut bytes = [0u8; 512];
        for i in 0..128 {
            let w = buf[i];
            bytes[i * 4 + 0] = (w & 0xFF) as u8;
            bytes[i * 4 + 1] = ((w >> 8) & 0xFF) as u8;
            bytes[i * 4 + 2] = ((w >> 16) & 0xFF) as u8;
            bytes[i * 4 + 3] = ((w >> 24) & 0xFF) as u8;
        }
        f.write_all(&bytes)
            .map_err(|e| BusError::Device(format!("write failed: {e}")))?;
    }
    Ok(())
}
