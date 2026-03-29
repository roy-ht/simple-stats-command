use sysinfo::System;

const BYTES_TO_GIB: f64 = 1.0 / (1024.0 * 1024.0 * 1024.0);

pub struct MemoryResult {
    pub used_gib: f64,
    pub total_gib: f64,
    pub percent: f64,
    pub swap_used_gib: f64,
    pub swap_total_gib: f64,
}

pub fn collect() -> MemoryResult {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory() as f64;
    let used = sys.used_memory() as f64;
    let swap_total = sys.total_swap() as f64;
    let swap_used = sys.used_swap() as f64;

    let percent = if total > 0.0 {
        (used / total) * 100.0
    } else {
        0.0
    };

    MemoryResult {
        used_gib: used * BYTES_TO_GIB,
        total_gib: total * BYTES_TO_GIB,
        percent,
        swap_used_gib: swap_used * BYTES_TO_GIB,
        swap_total_gib: swap_total * BYTES_TO_GIB,
    }
}
