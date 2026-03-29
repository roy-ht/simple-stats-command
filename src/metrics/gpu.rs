use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

const BYTES_TO_GIB: f64 = 1.0 / (1024.0 * 1024.0 * 1024.0);

pub struct GpuResult {
    pub temperature: Option<u32>,
    pub utilization: Option<u32>,
    pub mem_used_gib: Option<f64>,
    pub mem_total_gib: Option<f64>,
    pub name: Option<String>,
    pub cuda_version: Option<String>,
}

pub fn collect() -> Option<GpuResult> {
    let nvml = Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;

    let temperature = device.temperature(TemperatureSensor::Gpu).ok();
    let utilization = device.utilization_rates().ok().map(|u| u.gpu);
    let mem_info = device.memory_info().ok();
    let name = device.name().ok();
    let cuda_version = nvml.sys_cuda_driver_version().ok().map(|v| {
        let major = nvml_wrapper::cuda_driver_version_major(v);
        let minor = nvml_wrapper::cuda_driver_version_minor(v);
        format!("{major}.{minor}")
    });

    Some(GpuResult {
        temperature,
        utilization,
        mem_used_gib: mem_info.as_ref().map(|m| m.used as f64 * BYTES_TO_GIB),
        mem_total_gib: mem_info.as_ref().map(|m| m.total as f64 * BYTES_TO_GIB),
        name,
        cuda_version,
    })
}
