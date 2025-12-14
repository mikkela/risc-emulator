use crate::{
    bus::{BusResult, CpuBus},
    fp,
};

pub const ROM_START: u32 = 0xFFFF_F800;

// opcode IDs (samme rækkefølge som i C)
const MOV: u32 = 0;
const LSL: u32 = 1;
const ASR: u32 = 2;
const ROR: u32 = 3;
const AND: u32 = 4;
const ANN: u32 = 5;
const IOR: u32 = 6;
const XOR: u32 = 7;
const ADD: u32 = 8;
const SUB: u32 = 9;
const MUL: u32 = 10;
const DIV: u32 = 11;
const FAD: u32 = 12;
const FSB: u32 = 13;
const FML: u32 = 14;
const FDV: u32 = 15;

#[derive(Debug, Default)]
pub struct Cpu {
    pub pc: u32, // bytes
    pub r: [u32; 16],
    pub h: u32,
    pub z: bool,
    pub n: bool,
    pub c: bool,
    pub v: bool,
    pub progress: u32,
}

impl Cpu {
    pub fn reset(&mut self) {
        self.pc = ROM_START;
    }

    pub fn run<B: CpuBus>(&mut self, bus: &mut B, cycles: u32) -> BusResult<()> {
        self.progress = 20;
        for _ in 0..cycles {
            if self.progress == 0 {
                break;
            }
            self.step(bus)?;
        }
        Ok(())
    }

    #[inline]
    fn set_reg(&mut self, reg: usize, value: u32) {
        self.r[reg] = value;
        self.z = value == 0;
        self.n = (value as i32) < 0;
    }

    // progress-aware memory helpers
    #[inline]
    fn load_word<B: CpuBus>(&mut self, bus: &mut B, addr: u32) -> BusResult<u32> {
        bus.read_word_for_cpu(addr & !3, &mut self.progress)
    }

    #[inline]
    fn load_byte<B: CpuBus>(&mut self, bus: &mut B, addr: u32) -> BusResult<u8> {
        let w = self.load_word(bus, addr & !3)?;
        let shift = (addr & 3) * 8;
        Ok(((w >> shift) & 0xFF) as u8)
    }

    #[inline]
    fn store_word<B: CpuBus>(&mut self, bus: &mut B, addr: u32, value: u32) -> BusResult<()> {
        bus.write_word(addr & !3, value)
    }

    #[inline]
    fn store_byte<B: CpuBus>(&mut self, bus: &mut B, addr: u32, value: u8) -> BusResult<()> {
        let base = addr & !3;
        let mut w = self.load_word(bus, base)?;
        let shift = (addr & 3) * 8;
        w &= !(0xFFu32 << shift);
        w |= (value as u32) << shift;
        self.store_word(bus, base, w)
    }

    pub fn step<B: CpuBus>(&mut self, bus: &mut B) -> BusResult<()> {
        let ir = self.load_word(bus, self.pc)?;
        self.pc = self.pc.wrapping_add(4);

        let pbit = 0x8000_0000;
        let qbit = 0x4000_0000;
        let ubit = 0x2000_0000;
        let vbit = 0x1000_0000;

        if (ir & pbit) == 0 {
            // Register instructions
            let a = ((ir >> 24) & 0xF) as usize;
            let b = ((ir >> 20) & 0xF) as usize;
            let op = (ir >> 16) & 0xF;
            let im = (ir & 0xFFFF) as u32;
            let c = (ir & 0xF) as usize;

            let b_val = self.r[b];

            let c_val = if (ir & qbit) == 0 {
                self.r[c]
            } else if (ir & vbit) == 0 {
                im
            } else {
                0xFFFF_0000 | im
            };

            let mut a_val: u32;

            match op {
                MOV => {
                    if (ir & ubit) == 0 {
                        a_val = c_val;
                    } else if (ir & qbit) != 0 {
                        a_val = c_val << 16;
                    } else if (ir & vbit) != 0 {
                        a_val = 0xD0
                            | ((self.n as u32) << 31)
                            | ((self.z as u32) << 30)
                            | ((self.c as u32) << 29)
                            | ((self.v as u32) << 28);
                    } else {
                        a_val = self.h;
                    }
                }

                LSL => a_val = b_val << (c_val & 31),
                ASR => a_val = ((b_val as i32) >> (c_val & 31)) as u32,
                ROR => {
                    let sh = (c_val & 31) as u32;
                    a_val = (b_val >> sh) | (b_val << ((32 - sh) & 31));
                }

                AND => a_val = b_val & c_val,
                ANN => a_val = b_val & !c_val,
                IOR => a_val = b_val | c_val,
                XOR => a_val = b_val ^ c_val,

                ADD => {
                    a_val = b_val.wrapping_add(c_val);
                    if (ir & ubit) != 0 {
                        a_val = a_val.wrapping_add(self.c as u32);
                    }
                    self.c = a_val < b_val;
                    self.v = (((a_val ^ c_val) & (a_val ^ b_val)) >> 31) != 0;
                }

                SUB => {
                    a_val = b_val.wrapping_sub(c_val);
                    if (ir & ubit) != 0 {
                        a_val = a_val.wrapping_sub(self.c as u32);
                    }
                    self.c = a_val > b_val;
                    self.v = (((b_val ^ c_val) & (a_val ^ b_val)) >> 31) != 0;
                }

                MUL => {
                    if (ir & ubit) == 0 {
                        let tmp = (b_val as i32 as i64) * (c_val as i32 as i64);
                        a_val = tmp as u32;
                        self.h = ((tmp as u64) >> 32) as u32;
                    } else {
                        let tmp = (b_val as u64) * (c_val as u64);
                        a_val = tmp as u32;
                        self.h = (tmp >> 32) as u32;
                    }
                }

                DIV => {
                    if (c_val as i32) > 0 {
                        if (ir & ubit) == 0 {
                            let (q, r) = div_signed_c_positive(b_val as i32, c_val as i32);
                            a_val = q as u32;
                            self.h = r as u32;
                        } else {
                            if c_val == 0 {
                                a_val = 0;
                                self.h = b_val;
                            } else {
                                a_val = b_val / c_val;
                                self.h = b_val % c_val;
                            }
                        }
                    } else {
                        let d = fp::idiv(b_val, c_val, (ir & ubit) != 0);
                        a_val = d.quot;
                        self.h = d.rem;
                    }
                }

                FAD => a_val = fp::fp_add(b_val, c_val, (ir & ubit) != 0, (ir & vbit) != 0),
                FSB => a_val = fp::fp_add(b_val, c_val ^ 0x8000_0000, (ir & ubit) != 0, (ir & vbit) != 0),
                FML => a_val = fp::fp_mul(b_val, c_val),
                FDV => a_val = fp::fp_div(b_val, c_val),

                _ => unreachable!("invalid op"),
            }

            self.set_reg(a, a_val);
            Ok(())
        } else if (ir & qbit) == 0 {
            // Memory instructions
            let a = ((ir >> 24) & 0xF) as usize;
            let b = ((ir >> 20) & 0xF) as usize;

            let mut off = (ir & 0x000F_FFFF) as i32;
            off = (off ^ 0x0008_0000) - 0x0008_0000;

            let addr = self.r[b].wrapping_add(off as u32);

            if (ir & ubit) == 0 {
                let a_val = if (ir & vbit) == 0 {
                    self.load_word(bus, addr)?
                } else {
                    self.load_byte(bus, addr)? as u32
                };
                self.set_reg(a, a_val);
            } else {
                if (ir & vbit) == 0 {
                    self.store_word(bus, addr, self.r[a])?;
                } else {
                    self.store_byte(bus, addr, self.r[a] as u8)?;
                }
            }
            Ok(())
        } else {
            // Branch instructions
            let mut t = ((ir >> 27) & 1) != 0;
            match (ir >> 24) & 7 {
                0 => t ^= self.n,
                1 => t ^= self.z,
                2 => t ^= self.c,
                3 => t ^= self.v,
                4 => t ^= self.c || self.z,
                5 => t ^= self.n ^ self.v,
                6 => t ^= (self.n ^ self.v) || self.z,
                7 => t ^= true,
                _ => unreachable!(),
            }

            if t {
                if (ir & vbit) != 0 {
                    self.set_reg(15, self.pc);
                }

                if (ir & ubit) == 0 {
                    let c = (ir & 0xF) as usize;
                    self.pc = self.r[c];
                } else {
                    let mut off = (ir & 0x00FF_FFFF) as i32;
                    off = (off ^ 0x0080_0000) - 0x0080_0000;
                    let delta_bytes = (off as i64) * 4;
                    self.pc = self.pc.wrapping_add(delta_bytes as u32);
                }
            }
            Ok(())
        }
    }
}

fn div_signed_c_positive(b: i32, c: i32) -> (i32, i32) {
    debug_assert!(c > 0);
    if c == 0 {
        return (0, b);
    }
    let mut q = b / c;
    let mut r = b % c;
    if r < 0 {
        q -= 1;
        r += c;
    }
    (q, r)
}

#[derive(Debug, Clone, Copy)]
pub struct CpuView {
    pub pc: u32,
    pub r: [u32; 16],
    pub h: u32,
    pub z: bool,
    pub n: bool,
    pub c: bool,
    pub v: bool,
}

impl Cpu {
    pub fn view(&self) -> CpuView {
        CpuView {
            pc: self.pc,
            r: self.r,
            h: self.h,
            z: self.z,
            n: self.n,
            c: self.c,
            v: self.v,
        }
    }
}