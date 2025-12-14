use risc_emulator::Machine;

fn main() {
    let mut machine = Machine::new(1024, 768);

    machine.cpu.run(&mut machine.bus, 1_000).unwrap();
}