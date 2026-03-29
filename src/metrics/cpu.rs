use crate::cache::CacheData;

pub struct CpuResult {
    pub percent: Option<f64>,
    pub total_ticks: u64,
    pub idle_ticks: u64,
}

/// Collect CPU usage via cache-based delta.
/// On Linux: reads /proc/stat for raw tick counts.
/// On macOS: uses sysinfo single refresh (which returns instantaneous values).
pub fn collect(prev_cache: &Option<CacheData>, elapsed_secs: f64) -> CpuResult {
    let (total, idle) = read_cpu_ticks();

    let percent = if elapsed_secs > 0.0 {
        prev_cache.as_ref().and_then(|prev| {
            if prev.cpu_total_ticks == 0 {
                return None;
            }
            let total_delta = total.saturating_sub(prev.cpu_total_ticks);
            let idle_delta = idle.saturating_sub(prev.cpu_idle_ticks);
            if total_delta == 0 {
                return Some(0.0);
            }
            Some(((total_delta - idle_delta) as f64 / total_delta as f64) * 100.0)
        })
    } else {
        None
    };

    CpuResult {
        percent,
        total_ticks: total,
        idle_ticks: idle,
    }
}

/// Read raw CPU tick counts from the OS.
/// Returns (total_ticks, idle_ticks).
#[cfg(target_os = "linux")]
fn read_cpu_ticks() -> (u64, u64) {
    // /proc/stat first line: cpu  user nice system idle iowait irq softirq steal guest guest_nice
    let Ok(content) = std::fs::read_to_string("/proc/stat") else {
        return (0, 0);
    };
    let Some(line) = content.lines().next() else {
        return (0, 0);
    };
    let vals: Vec<u64> = line
        .split_whitespace()
        .skip(1) // skip "cpu"
        .filter_map(|s| s.parse().ok())
        .collect();

    if vals.len() < 4 {
        return (0, 0);
    }

    let total: u64 = vals.iter().sum();
    // idle = idle + iowait
    let idle = vals[3] + vals.get(4).unwrap_or(&0);
    (total, idle)
}

#[cfg(target_os = "macos")]
fn read_cpu_ticks() -> (u64, u64) {
    // Use host_processor_info Mach API to get actual monotonically-increasing
    // CPU tick counts (user + system + idle + nice per CPU).
    // These accumulate over time, making the cache-based delta calculation work correctly.
    use std::mem;

    type MachPort = u32;
    type KernReturn = i32;
    type ProcessorFlavor = i32;
    type MachMsgTypeNumberT = u32;
    type ProcessorInfoArrayT = *mut i32;

    const PROCESSOR_CPU_LOAD_INFO: ProcessorFlavor = 2;
    const CPU_STATE_USER: usize = 0;
    const CPU_STATE_SYSTEM: usize = 1;
    const CPU_STATE_IDLE: usize = 2;
    const CPU_STATE_NICE: usize = 3;
    const CPU_STATE_MAX: usize = 4;
    const KERN_SUCCESS: KernReturn = 0;

    #[link(name = "System", kind = "framework")]
    unsafe extern "C" {
        fn mach_host_self() -> MachPort;
        fn mach_task_self() -> MachPort;
        fn host_processor_info(
            host: MachPort,
            flavor: ProcessorFlavor,
            out_processor_count: *mut u32,
            out_processor_info: *mut ProcessorInfoArrayT,
            out_processor_info_count: *mut MachMsgTypeNumberT,
        ) -> KernReturn;
        fn vm_deallocate(target_task: MachPort, address: *const i32, size: usize) -> KernReturn;
    }

    unsafe {
        let host = mach_host_self();
        let mut cpu_count: u32 = 0;
        let mut cpu_info: ProcessorInfoArrayT = std::ptr::null_mut();
        let mut cpu_info_count: MachMsgTypeNumberT = 0;

        let ret = host_processor_info(
            host,
            PROCESSOR_CPU_LOAD_INFO,
            &mut cpu_count,
            &mut cpu_info,
            &mut cpu_info_count,
        );

        if ret != KERN_SUCCESS || cpu_info.is_null() {
            return (0, 0);
        }

        let mut total_ticks = 0u64;
        let mut idle_ticks = 0u64;

        for i in 0..cpu_count as usize {
            let base = i * CPU_STATE_MAX;
            let user = *cpu_info.add(base + CPU_STATE_USER) as u64;
            let system = *cpu_info.add(base + CPU_STATE_SYSTEM) as u64;
            let idle = *cpu_info.add(base + CPU_STATE_IDLE) as u64;
            let nice = *cpu_info.add(base + CPU_STATE_NICE) as u64;
            total_ticks += user + system + idle + nice;
            idle_ticks += idle;
        }

        let _ = vm_deallocate(
            mach_task_self(),
            cpu_info,
            cpu_info_count as usize * mem::size_of::<i32>(),
        );

        (total_ticks, idle_ticks)
    }
}
