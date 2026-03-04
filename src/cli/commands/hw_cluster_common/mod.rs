/// LCM (Least Common Multiple) used to normalize memory
/// capacity values reported by hardware inventory. Memory
/// DIMMs come in multiples of 16 GiB (16384 MiB), so this
/// value is used to bucket nodes by memory capacity.
pub const MEMORY_CAPACITY_LCM: u64 = 16384; // 1024 * 16

pub mod command;
pub mod utils;
