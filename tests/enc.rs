pub fn reg(op: u32, a: u32, b: u32, c: u32, q: bool, u: bool, v: bool, imm16: u32) -> u32 {
    let mut ir = 0u32; // p=0
    if q { ir |= 0x4000_0000; }
    if u { ir |= 0x2000_0000; }
    if v { ir |= 0x1000_0000; }
    ir |= (a & 0xF) << 24;
    ir |= (b & 0xF) << 20;
    ir |= (op & 0xF) << 16;
    ir |= imm16 & 0xFFFF;
    ir |= c & 0xF;
    ir
}

pub fn mem(a: u32, b: u32, off20: i32, store: bool, byte: bool) -> u32 {
    let mut ir = 0x8000_0000; // p=1
    // q=0
    if store { ir |= 0x2000_0000; } // u
    if byte  { ir |= 0x1000_0000; } // v
    ir |= (a & 0xF) << 24;
    ir |= (b & 0xF) << 20;

    let off = (off20 as i32) & 0x000F_FFFF;
    ir |= off as u32;
    ir
}

pub fn br(cond: u32, t: bool, u_rel: bool, v_link: bool, c_reg: u32, off24_words: i32) -> u32 {
    let mut ir = 0xC000_0000; // p=1 q=1
    if t { ir |= 1 << 27; }
    ir |= (cond & 7) << 24;
    if u_rel { ir |= 0x2000_0000; }
    if v_link { ir |= 0x1000_0000; }

    if u_rel {
        let off = (off24_words as i32) & 0x00FF_FFFF;
        ir |= off as u32;
    } else {
        ir |= c_reg & 0xF;
    }
    ir
}
