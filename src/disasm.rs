// src/disasm.rs
//
// Disassembler for Project Oberon / RISC (register/memory/branch formats)
// PC/address is in BYTES. (Your CPU uses pc in bytes.)
//
// This is meant for debugger UI: readable output, not perfect assembly syntax.

#[derive(Debug, Clone, Copy)]
pub enum InstrKind {
    Reg,
    Mem,
    Branch,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DisasmLine {
    pub addr: u32,         // byte address of instruction
    pub raw: u32,          // instruction word
    pub text: String,      // formatted disassembly
    pub kind: InstrKind,
    pub branch_target: Option<u32>,
}

fn sign_extend_20(x: u32) -> i32 {
    let mut off = (x & 0x000F_FFFF) as i32;
    off = (off ^ 0x0008_0000) - 0x0008_0000;
    off
}

fn sign_extend_24(x: u32) -> i32 {
    let mut off = (x & 0x00FF_FFFF) as i32;
    off = (off ^ 0x0080_0000) - 0x0080_0000;
    off
}

fn imm16(ir: u32, v: bool) -> i32 {
    let im = (ir & 0xFFFF) as u32;
    if !v {
        im as i32
    } else {
        (0xFFFF_0000u32 | im) as i32
    }
}

fn op_name(op: u32) -> &'static str {
    match op {
        0 => "MOV",
        1 => "LSL",
        2 => "ASR",
        3 => "ROR",
        4 => "AND",
        5 => "ANN",
        6 => "IOR",
        7 => "XOR",
        8 => "ADD",
        9 => "SUB",
        10 => "MUL",
        11 => "DIV",
        12 => "FAD",
        13 => "FSB",
        14 => "FML",
        15 => "FDV",
        _ => "???",
    }
}

fn cond_name(cond: u32, t_invert: bool) -> &'static str {
    // In C: bool t = (ir>>27)&1; then t ^= <flag-expression>; if (t) branch.
    // So: if t_invert==0 => branch when expr==1
    //     if t_invert==1 => branch when expr==0
    match (cond, t_invert) {
        (0, false) => "BN",     // N==1
        (0, true)  => "BNN",    // N==0

        (1, false) => "BZ",     // Z==1
        (1, true)  => "BNZ",    // Z==0

        (2, false) => "BC",     // C==1
        (2, true)  => "BNC",    // C==0

        (3, false) => "BV",     // V==1
        (3, true)  => "BNV",    // V==0

        (4, false) => "BLS",    // (C|Z)==1
        (4, true)  => "BHI",    // (C|Z)==0   (inverse of LS)

        (5, false) => "BLT",    // (N^V)==1
        (5, true)  => "BGE",    // (N^V)==0

        (6, false) => "BLE",    // (N^V)|Z
        (6, true)  => "BGT",    // inverse

        (7, false) => "B",      // always
        (7, true)  => "BNEVER", // never (inverse of always)
        _ => "B?",
    }
}

fn reg_name(r: u32) -> String {
    format!("R{}", r)
}

fn fmt_imm(i: i32) -> String {
    if i < 0 {
        format!("-0x{:X}", (-i) as u32)
    } else {
        format!("0x{:X}", i as u32)
    }
}

pub fn disassemble_at(addr: u32, ir: u32) -> DisasmLine {
    let p = (ir & 0x8000_0000) != 0;
    let q = (ir & 0x4000_0000) != 0;
    let u = (ir & 0x2000_0000) != 0;
    let v = (ir & 0x1000_0000) != 0;

    if !p {
        // Register format
        let a  = (ir >> 24) & 0xF;
        let b  = (ir >> 20) & 0xF;
        let op = (ir >> 16) & 0xF;
        let im = ir & 0xFFFF;
        let c  = ir & 0xF;

        let mnemonic = op_name(op);

        // Operand C: reg OR imm16 (unsigned/signed)
        let c_str = if !q {
            reg_name(c)
        } else {
            let imm = imm16(ir, v);
            fmt_imm(imm)
        };

        // Special-case MOV (because the ISA uses u/q/v to select variants)
        let text = if op == 0 {
            // MOV variants from your C:
            // u=0: a = c_val (reg or imm16)
            // u=1 & q=1: a = imm16 << 16  (upper immediate)
            // u=1 & q=0 & v=1: a = flags (NZCV) encoded
            // u=1 & q=0 & v=0: a = H
            if !u {
                // MOV
                if !q {
                    format!("MOV  {}, {}", reg_name(a), c_str)
                } else {
                    format!("MOV  {}, {}", reg_name(a), c_str)
                }
            } else if q {
                // MOV imm<<16 (upper)
                let immu = im as u32;
                format!("MOVH {}, 0x{:04X}<<16", reg_name(a), immu)
            } else if v {
                // flags -> reg
                format!("MOVF {}, NZCV", reg_name(a))
            } else {
                // H -> reg
                format!("MOVH {}, H", reg_name(a))
            }
        } else {
            // Normal reg ops
            match op {
                8 => { // ADD / ADC (u adds carry)
                    if u {
                        format!("ADC  {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    } else {
                        format!("ADD  {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    }
                }
                9 => { // SUB / SBC (u subtracts carry/borrow)
                    if u {
                        format!("SBC  {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    } else {
                        format!("SUB  {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    }
                }
                10 => { // MUL signed/unsigned
                    if u {
                        format!("MULU {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    } else {
                        format!("MULS {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    }
                }
                11 => { // DIV signed/unsigned
                    if u {
                        format!("DIVU {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    } else {
                        format!("DIVS {}, {}, {}", reg_name(a), reg_name(b), c_str)
                    }
                }
                12 | 13 => { // FAD/FSB with u/v modes
                    // We don’t decode the float-mode fully here; we show flags.
                    let suffix = match (u, v) {
                        (false, false) => "",
                        (true,  false) => ".U",
                        (false, true)  => ".V",
                        (true,  true)  => ".UV",
                    };
                    format!("{mnemonic}{suffix} {}, {}, {}", reg_name(a), reg_name(b), c_str)
                }
                14 | 15 => {
                    format!("{mnemonic} {}, {}, {}", reg_name(a), reg_name(b), c_str)
                }
                _ => {
                    format!("{mnemonic} {}, {}, {}", reg_name(a), reg_name(b), c_str)
                }
            }
        };

        return DisasmLine {
            addr,
            raw: ir,
            text,
            kind: InstrKind::Reg,
            branch_target: None,
        };
    }

    if !q {
        // Memory format
        let a = (ir >> 24) & 0xF;
        let b = (ir >> 20) & 0xF;
        let off = sign_extend_20(ir);
        let off_str = if off == 0 {
            "".to_string()
        } else {
            format!("+{}", fmt_imm(off))
        };

        let (mnemonic, operands) = if !u {
            // load
            if !v {
                ("LDW", format!("{}, [{}{}]", reg_name(a), reg_name(b), off_str))
            } else {
                ("LDB", format!("{}, [{}{}]", reg_name(a), reg_name(b), off_str))
            }
        } else {
            // store
            if !v {
                ("STW", format!("{}, [{}{}]", reg_name(a), reg_name(b), off_str))
            } else {
                ("STB", format!("{}, [{}{}]", reg_name(a), reg_name(b), off_str))
            }
        };

        return DisasmLine {
            addr,
            raw: ir,
            text: format!("{mnemonic} {operands}"),
            kind: InstrKind::Mem,
            branch_target: None,
        };
    }

    // Branch format
    let t_invert = ((ir >> 27) & 1) != 0;
    let cond = (ir >> 24) & 7;
    let link = v;

    let mnem = cond_name(cond, t_invert);

    let (target, ops) = if !u {
        // register target
        let c = ir & 0xF;
        (None, format!("{}", reg_name(c)))
    } else {
        // relative target: PC is bytes; in C they do risc->PC is in words.
        // In your byte-PC emulator, we assume addr is current instruction address.
        // Branch uses "PC after fetch" in C; your CPU likely already incremented PC before applying off.
        // For disasm we show target = addr + 4 + off*4?  In C: risc->PC (word index) already incremented.
        // Here we approximate as: next = addr + 4; target = next + off*4 (because off is in words).
        let off = sign_extend_24(ir);
        let next = addr.wrapping_add(4);
        let target = next.wrapping_add((off as i64 * 4) as u32);
        (Some(target), format!("{}", fmt_imm(off)))
    };

    let text = if link {
        if let Some(tgt) = target {
            format!("{mnem}.L {ops}  ; -> 0x{tgt:08X}")
        } else {
            format!("{mnem}.L {ops}")
        }
    } else if let Some(tgt) = target {
        format!("{mnem} {ops}  ; -> 0x{tgt:08X}")
    } else {
        format!("{mnem} {ops}")
    };

    DisasmLine {
        addr,
        raw: ir,
        text,
        kind: InstrKind::Branch,
        branch_target: target,
    }
}

/// Convenience formatter for GUI: includes addr + raw + decoded.
pub fn format_line(addr: u32, ir: u32) -> String {
    let d = disassemble_at(addr, ir);
    format!("0x{addr:08X}:  0x{ir:08X}  {}", d.text)
}
