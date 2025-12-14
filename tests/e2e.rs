use risc_emulator::{Machine, cpu::Cpu};

use risc_emulator::machine::ROM_START;
use risc_emulator::machine::IO_START;

// genbrug encoder fra tests/enc.rs (kopiér minimal her)
fn reg(op: u32, a: u32, b: u32, c: u32, q: bool, u: bool, v: bool, imm16: u32) -> u32 {
    let mut ir = 0u32;
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

fn mem(a: u32, b: u32, off20: i32, store: bool, byte: bool) -> u32 {
    let mut ir = 0x8000_0000;
    if store { ir |= 0x2000_0000; }
    if byte  { ir |= 0x1000_0000; }
    ir |= (a & 0xF) << 24;
    ir |= (b & 0xF) << 20;
    let off = (off20 as i32) & 0x000F_FFFF;
    ir |= off as u32;
    ir
}

const MOV: u32 = 0;

#[test]
fn e2e_framebuffer_write_updates_damage() {
    // Lille memory map så framebuffer starter tidligt
    // mem_size = 0x400, display_start = 0x200
    // fb_width_words=8, fb_height=8 (bare noget)
    let display_start = 0x200;
    let mem_size = 0x400;

    let prog = vec![
        reg(MOV, 1, 0, 0, true, false, false, display_start as u32),
        reg(MOV, 2, 0, 0, true, true,  false, 0xABCD),
        mem(2, 1, 0, true, false),

        // Branch always, relative -1 word => loop på sig selv (holder PC inde i ROM)
        0xE7FF_FFFF,
    ];

    let mut m = Machine::new_for_tests(prog, mem_size, display_start, 8, 8);

    m.cpu.progress = 1000;
    m.cpu.run(&mut m.bus, 10).unwrap();

    let dmg = m.bus.reset_damage();
    // Vi skrev mindst én word i fb => damage må være “gyldigt” (x2>=x1 osv.)
    assert!(dmg.x2 >= dmg.x1);
    assert!(dmg.y2 >= dmg.y1);

    // sanity: IO/ROM ligger stadig højt, men irrelevant – her sikrer vi bare at test kører end-to-end.
    assert!(ROM_START > mem_size);
    assert!(IO_START > ROM_START);
}
