pub mod cpu;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod memory;
pub mod network;

use std::collections::HashMap;

use crate::cache::{self, CacheData};
use crate::format::NeededMetrics;

pub struct CollectOptions<'a> {
    pub needed: &'a NeededMetrics,
    pub interface: Option<&'a str>,
    pub cache_path: &'a str,
}

pub struct CollectResult {
    pub values: HashMap<String, f64>,
    pub str_values: HashMap<String, String>,
    pub na_keys: Vec<String>,
}

pub fn collect_all(opts: &CollectOptions) -> CollectResult {
    let mut values = HashMap::new();
    let mut str_values: HashMap<String, String> = HashMap::new();
    let mut na_keys: Vec<String> = Vec::new();

    let prev_cache = cache::read_cache(opts.cache_path);
    let now_ms = CacheData::now_ms();

    let mut new_cache = CacheData {
        timestamp_ms: now_ms,
        ..Default::default()
    };

    let elapsed_secs = prev_cache
        .as_ref()
        .map(|c| (now_ms.saturating_sub(c.timestamp_ms)) as f64 / 1000.0)
        .unwrap_or(0.0);

    if opts.needed.cpu {
        let cpu_val = cpu::collect(&prev_cache, elapsed_secs);
        new_cache.cpu_total_ticks = cpu_val.total_ticks;
        new_cache.cpu_idle_ticks = cpu_val.idle_ticks;
        if let Some(pct) = cpu_val.percent {
            values.insert("cpu".to_string(), pct);
        } else {
            na_keys.push("cpu".to_string());
        }
    }

    if opts.needed.memory {
        let mem = memory::collect();
        values.insert("mem_used".to_string(), mem.used_gib);
        values.insert("mem_total".to_string(), mem.total_gib);
        values.insert("mem_percent".to_string(), mem.percent);
        values.insert("swap_used".to_string(), mem.swap_used_gib);
        values.insert("swap_total".to_string(), mem.swap_total_gib);
    }

    if opts.needed.network {
        let net = network::collect(opts.interface);
        new_cache.net_rx_bytes = net.rx_bytes;
        new_cache.net_tx_bytes = net.tx_bytes;

        let has_prev_net = prev_cache
            .as_ref()
            .is_some_and(|c| elapsed_secs > 0.0 && (c.net_rx_bytes > 0 || c.net_tx_bytes > 0));

        if has_prev_net {
            let prev = prev_cache.as_ref().unwrap();
            let down = (net.rx_bytes.saturating_sub(prev.net_rx_bytes)) as f64
                / elapsed_secs
                / 1_000_000.0;
            let up = (net.tx_bytes.saturating_sub(prev.net_tx_bytes)) as f64
                / elapsed_secs
                / 1_000_000.0;
            values.insert("net_down".to_string(), down);
            values.insert("net_up".to_string(), up);
        } else {
            na_keys.push("net_down".to_string());
            na_keys.push("net_up".to_string());
        }
    }

    if opts.needed.gpu {
        collect_gpu(&mut values, &mut str_values, &mut na_keys);
    }

    cache::write_cache(opts.cache_path, &new_cache);

    CollectResult {
        values,
        str_values,
        na_keys,
    }
}

#[cfg(feature = "gpu")]
fn collect_gpu(
    values: &mut HashMap<String, f64>,
    str_values: &mut HashMap<String, String>,
    na_keys: &mut Vec<String>,
) {
    match gpu::collect() {
        Some(g) => {
            insert_or_na(values, na_keys, "gpu_temp", g.temperature.map(|v| v as f64));
            insert_or_na(values, na_keys, "gpu_util", g.utilization.map(|v| v as f64));
            insert_or_na(values, na_keys, "gpu_mem_used", g.mem_used_gib);
            insert_or_na(values, na_keys, "gpu_mem_total", g.mem_total_gib);
            insert_str_or_na(str_values, na_keys, "gpu_name", g.name);
            insert_str_or_na(str_values, na_keys, "cuda_ver", g.cuda_version);
        }
        None => {
            for key in &["gpu_temp", "gpu_util", "gpu_mem_used", "gpu_mem_total", "gpu_name", "cuda_ver"] {
                na_keys.push(key.to_string());
            }
        }
    }
}

#[cfg(feature = "gpu")]
fn insert_or_na(
    values: &mut HashMap<String, f64>,
    na_keys: &mut Vec<String>,
    key: &str,
    val: Option<f64>,
) {
    match val {
        Some(v) => { values.insert(key.to_string(), v); }
        None => { na_keys.push(key.to_string()); }
    }
}

#[cfg(feature = "gpu")]
fn insert_str_or_na(
    str_values: &mut HashMap<String, String>,
    na_keys: &mut Vec<String>,
    key: &str,
    val: Option<String>,
) {
    match val {
        Some(v) => { str_values.insert(key.to_string(), v); }
        None => { na_keys.push(key.to_string()); }
    }
}

#[cfg(not(feature = "gpu"))]
fn collect_gpu(
    _values: &mut HashMap<String, f64>,
    _str_values: &mut HashMap<String, String>,
    na_keys: &mut Vec<String>,
) {
    for key in &[
        "gpu_temp",
        "gpu_util",
        "gpu_mem_used",
        "gpu_mem_total",
        "gpu_name",
        "cuda_ver",
    ] {
        na_keys.push(key.to_string());
    }
}
