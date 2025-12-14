mod enc;
mod test_bus;

use enc::{br, mem, reg};
use risc_emulator::cpu::Cpu;
use test_bus::{TestBus, ROM_START};

const MOV: u32 = 0;
const IOR: u32 = 6;
const ADD: u32 = 8;

fn run_prog(bus: &mut TestBus, cpu: &mut Cpu, prog: &[u32], steps: usize) {
    bus.rom[..prog.len()].copy_from_slice(prog);
    cpu.pc = ROM_START;
    cpu.progress = 1_000;
    for _ in 0..steps {
        cpu.step(bus).unwrap();
    }
}

#[test]
fn unit_add_and_flags() {
    // R1=1, R2=2, R0=R1+R2 => 3 (Z=false, N=false)
    let prog = [
        reg(MOV, 1, 0, 0, true, false, false, 1),
        reg(MOV, 2, 0, 0, true, false, false, 2),
        reg(ADD, 0, 1, 2, false, false, false, 0),
    ];

    let mut bus = TestBus::new(1024, 512);
    let mut cpu = Cpu::default();
    run_prog(&mut bus, &mut cpu, &prog, prog.len());

    assert_eq!(cpu.r[0], 3);
    assert!(!cpu.z);
    assert!(!cpu.n);
}

#[test]
fn unit_store_then_load_word() {
    // Build 0x11223344 in R2 via: MOV hi<<16 + OR lo
    // R1=0x100
    // [R1] = R2
    // R0 = [R1]
    let prog = [
        reg(MOV, 1, 0, 0, true, false, false, 0x0100),          // R1 = 0x0100
        reg(MOV, 2, 0, 0, true, true,  false, 0x1122),          // MOV (u=1,q=1) => imm<<16 => 0x1122_0000
        reg(IOR, 2, 2, 0, true, false, false, 0x3344),          // R2 = R2 | 0x3344 => 0x1122_3344
        mem(2, 1, 0, true, false),                              // store word [R1+0] = R2
        mem(0, 1, 0, false, false),                             // load word R0 = [R1+0]
    ];

    let mut bus = TestBus::new(2048, 512);
    let mut cpu = Cpu::default();
    run_prog(&mut bus, &mut cpu, &prog, prog.len());

    assert_eq!(cpu.r[0], 0x1122_3344);
    assert_eq!(bus.ram[(0x0100 / 4) as usize], 0x1122_3344);
}

#[test]
fn unit_branch_relative_and_link() {
    // MOV R0, #0
    // branch always + link + rel +1 word => skip MOV #123
    // MOV R0, #123  (skipped)
    // MOV R0, #7    (taken)
    let prog = [
        reg(MOV, 0, 0, 0, true, false, false, 0),
        br(7, false, true, true, 0, 1),
        reg(MOV, 0, 0, 0, true, false, false, 123),
        reg(MOV, 0, 0, 0, true, false, false, 7),
    ];

    let mut bus = TestBus::new(1024, 512);
    let mut cpu = Cpu::default();
    run_prog(&mut bus, &mut cpu, &prog, prog.len()); // kør “forbi” hele programmet

    assert_eq!(cpu.r[0], 7);

    // Link-reg = pc lige efter branch-instr fetch (dvs. adressen på den “skippede” instr)
    // instr0 @ ROM_START
    // instr1 (branch) @ ROM_START+4, efter fetch pc = ROM_START+8
    assert_eq!(cpu.r[15], ROM_START + 8);
}
