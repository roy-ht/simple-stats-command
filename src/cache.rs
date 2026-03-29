use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// Fixed-size binary cache: 5 x u64 = 40 bytes
const CACHE_SIZE: usize = 40;

#[derive(Default)]
pub struct CacheData {
    pub timestamp_ms: u64,
    pub cpu_total_ticks: u64,
    pub cpu_idle_ticks: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

impl CacheData {
    pub fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn to_bytes(&self) -> [u8; CACHE_SIZE] {
        let mut buf = [0u8; CACHE_SIZE];
        buf[0..8].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        buf[8..16].copy_from_slice(&self.cpu_total_ticks.to_le_bytes());
        buf[16..24].copy_from_slice(&self.cpu_idle_ticks.to_le_bytes());
        buf[24..32].copy_from_slice(&self.net_rx_bytes.to_le_bytes());
        buf[32..40].copy_from_slice(&self.net_tx_bytes.to_le_bytes());
        buf
    }

    fn from_bytes(buf: &[u8; CACHE_SIZE]) -> Self {
        Self {
            timestamp_ms: u64::from_le_bytes(buf[0..8].try_into().unwrap()),
            cpu_total_ticks: u64::from_le_bytes(buf[8..16].try_into().unwrap()),
            cpu_idle_ticks: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            net_rx_bytes: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            net_tx_bytes: u64::from_le_bytes(buf[32..40].try_into().unwrap()),
        }
    }
}

pub fn default_cache_path() -> String {
    let dir = std::env::temp_dir();
    format!("{}/simple-stats.bin", dir.display())
}

pub fn read_cache(path: &str) -> Option<CacheData> {
    let data = fs::read(path).ok()?;
    if data.len() != CACHE_SIZE {
        return None;
    }
    Some(CacheData::from_bytes(data.as_slice().try_into().ok()?))
}

pub fn write_cache(path: &str, data: &CacheData) {
    let bytes = data.to_bytes();
    let tmp_path = format!("{path}.tmp");
    if fs::write(&tmp_path, bytes).is_ok() {
        let _ = fs::rename(&tmp_path, path);
    }
}
