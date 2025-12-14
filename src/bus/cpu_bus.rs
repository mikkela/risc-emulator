use crate::bus::BusResult;

/// CPU-bus er *progress-aware* på reads (så vi matcher din C progress-- heuristik).
pub trait CpuBus {
    fn read_word_for_cpu(&mut self, addr: u32, progress: &mut u32) -> BusResult<u32>;
    fn write_word(&mut self, addr: u32, value: u32) -> BusResult<()>;
}