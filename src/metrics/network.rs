use sysinfo::Networks;

pub struct NetworkResult {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub fn collect(interface: Option<&str>) -> NetworkResult {
    let networks = Networks::new_with_refreshed_list();

    let mut rx: u64 = 0;
    let mut tx: u64 = 0;

    for (name, data) in &networks {
        if let Some(iface) = interface {
            if name != iface {
                continue;
            }
        }
        rx += data.total_received();
        tx += data.total_transmitted();
    }

    NetworkResult {
        rx_bytes: rx,
        tx_bytes: tx,
    }
}
