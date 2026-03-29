use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::error::NvmlError;

const BYTES_TO_GIB: f64 = 1.0 / (1024.0 * 1024.0 * 1024.0);

pub struct GpuResult {
    pub temperature: Option<u32>,
    pub utilization: Option<u32>,
    pub mem_used_gib: Option<f64>,
    pub mem_total_gib: Option<f64>,
    pub name: Option<String>,
    pub cuda_version: Option<String>,
    pub debug_log: Vec<String>,
}

pub fn collect() -> Option<GpuResult> {
    let mut log: Vec<String> = Vec::new();

    let nvml = match Nvml::init() {
        Ok(n) => n,
        Err(e) => {
            log.push(format!("[gpu] Nvml::init() failed: {e:?}"));
            return Some(GpuResult::unavailable(log));
        }
    };
    log.push("[gpu] Nvml::init() ok".to_string());

    let device = match nvml.device_by_index(0) {
        Ok(d) => d,
        Err(e) => {
            log.push(format!("[gpu] device_by_index(0) failed: {e:?}"));
            return Some(GpuResult::unavailable(log));
        }
    };
    log.push("[gpu] device_by_index(0) ok".to_string());

    let temperature = match device.temperature(TemperatureSensor::Gpu) {
        Ok(v) => { log.push(format!("[gpu] temperature: {v}")); Some(v) }
        Err(e) => { log.push(format!("[gpu] temperature failed: {e:?}")); None }
    };

    let utilization = match device.utilization_rates() {
        Ok(u) => { log.push(format!("[gpu] utilization: {}", u.gpu)); Some(u.gpu) }
        Err(e) => { log.push(format!("[gpu] utilization_rates failed: {e:?}")); None }
    };

    // nvml-wrapper 0.12 uses nvmlDeviceGetMemoryInfo_v2 internally.
    // - FailedToLoadSymbol: older driver without v2 symbol → fall back to v1 API
    // - NotSupported: unified memory device (e.g. GB10/Grace Blackwell) → use host memory
    let (mem_used, mem_total) = match device.memory_info() {
        Ok(m) => {
            log.push(format!("[gpu] memory_info (v2) ok: used={} total={}", m.used, m.total));
            (Some(m.used), Some(m.total))
        }
        Err(NvmlError::FailedToLoadSymbol(ref sym)) => {
            log.push(format!("[gpu] memory_info (v2) FailedToLoadSymbol: {sym} — trying v1"));
            let (u, t, v1_log) = memory_info_v1(&nvml, &device);
            log.extend(v1_log);
            (u, t)
        }
        Err(NvmlError::NotSupported) => {
            log.push("[gpu] memory_info (v2) NotSupported — unified memory device, using host memory".to_string());
            let (u, t, host_log) = memory_info_from_host();
            log.extend(host_log);
            (u, t)
        }
        Err(e) => {
            log.push(format!("[gpu] memory_info (v2) failed: {e:?}"));
            (None, None)
        }
    };

    let name = match device.name() {
        Ok(n) => { log.push(format!("[gpu] name: {n}")); Some(n) }
        Err(e) => { log.push(format!("[gpu] name failed: {e:?}")); None }
    };

    let cuda_version = match nvml.sys_cuda_driver_version() {
        Ok(v) => {
            let major = nvml_wrapper::cuda_driver_version_major(v);
            let minor = nvml_wrapper::cuda_driver_version_minor(v);
            let s = format!("{major}.{minor}");
            log.push(format!("[gpu] cuda_version: {s}"));
            Some(s)
        }
        Err(e) => { log.push(format!("[gpu] sys_cuda_driver_version failed: {e:?}")); None }
    };

    Some(GpuResult {
        temperature,
        utilization,
        mem_used_gib: mem_used.map(|v| v as f64 * BYTES_TO_GIB),
        mem_total_gib: mem_total.map(|v| v as f64 * BYTES_TO_GIB),
        name,
        cuda_version,
        debug_log: log,
    })
}

impl GpuResult {
    fn unavailable(log: Vec<String>) -> Self {
        Self {
            temperature: None,
            utilization: None,
            mem_used_gib: None,
            mem_total_gib: None,
            name: None,
            cuda_version: None,
            debug_log: log,
        }
    }
}

/// Fallback for unified memory devices (e.g. GB10/Grace Blackwell) where
/// nvmlDeviceGetMemoryInfo returns NotSupported because GPU and CPU share host memory.
fn memory_info_from_host() -> (Option<u64>, Option<u64>, Vec<String>) {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_memory();
    let total = sys.total_memory();
    let used = sys.used_memory();
    let log = vec![format!(
        "[gpu] memory_info (host fallback): used={used} total={total}"
    )];
    (Some(used), Some(total), log)
}

/// Fallback for drivers that do not export nvmlDeviceGetMemoryInfo_v2.
/// Returns (used, total, log_lines).
fn memory_info_v1(
    nvml: &Nvml,
    device: &nvml_wrapper::Device,
) -> (Option<u64>, Option<u64>, Vec<String>) {
    use nvml_wrapper_sys::bindings::nvmlMemory_t;
    use std::mem;

    const NVML_SUCCESS: u32 = 0;

    let lib = nvml.lib();
    let raw_device = unsafe { device.handle() };
    let mut info: nvmlMemory_t = unsafe { mem::zeroed() };
    let ret = unsafe { lib.nvmlDeviceGetMemoryInfo(raw_device, &mut info) };

    if ret == NVML_SUCCESS {
        let log = vec![format!(
            "[gpu] memory_info (v1) ok: used={} total={}",
            info.used, info.total
        )];
        (Some(info.used), Some(info.total), log)
    } else {
        let log = vec![format!("[gpu] memory_info (v1) failed: nvmlReturn={ret}")];
        (None, None, log)
    }
}
