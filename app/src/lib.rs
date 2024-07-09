mod allocator;

use serde::Serialize;
use sysinfo::System;

#[tauri::command]
fn sysinfo() -> System {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys
}

#[derive(Serialize)]
pub struct GpuInfo {
    adapter:      wgpu::AdapterInfo,
    features:     wgpu::Features,
    limits:       wgpu::Limits,
    capabilities: wgpu::DownlevelCapabilities,
}

#[tauri::command]
async fn gpuinfo() -> Vec<GpuInfo> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    instance
        .enumerate_adapters(wgpu::Backends::all())
        .into_iter()
        .map(|adapter| GpuInfo {
            adapter:      adapter.get_info(),
            features:     adapter.features(),
            limits:       adapter.limits(),
            capabilities: adapter.get_downlevel_capabilities(),
        })
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![sysinfo, gpuinfo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
