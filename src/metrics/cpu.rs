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
    use sysinfo::System;
    // On macOS, sysinfo can compute CPU usage from a single refresh
    // using host_processor_info Mach API which gives instantaneous values.
    // We store the usage as "ticks" (percent * 100 for total=10000)
    // to fit the delta computation model.
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    let usage = sys.global_cpu_usage() as f64;
    // Encode as synthetic ticks: total=10000, idle=10000*(1-usage/100)
    let total = 10000u64;
    let idle = ((100.0 - usage) * 100.0) as u64;
    (total, idle)
}
