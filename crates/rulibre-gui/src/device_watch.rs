use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use rulibre_core::device::{self, Device, DeviceEvent};
use tauri::{AppHandle, Emitter};

pub async fn watch_loop(app: AppHandle, shared: Arc<Mutex<Option<Device>>>) {
    let mut was_connected = false;
    loop {
        let detected = device::detect_device();
        match (&detected, was_connected) {
            (Some(dev), false) => {
                *shared.lock().unwrap() = Some(dev.clone());
                let _ = app.emit("device-event", DeviceEvent::Connected(dev.clone()));
                was_connected = true;
            }
            (None, true) => {
                *shared.lock().unwrap() = None;
                let _ = app.emit("device-event", DeviceEvent::Disconnected);
                was_connected = false;
            }
            _ => {}
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
