use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::types::{Book, Config, ConvertOk, Device, DeviceEvent, Metadata};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = "listen")]
    async fn listen_event(event: &str, handler: &js_sys::Function) -> JsValue;
}

async fn call<R: for<'de> Deserialize<'de>>(cmd: &str, args: impl Serialize) -> R {
    let args = to_value(&args).expect("serialize args");
    let result = invoke(cmd, args).await;
    from_value(result).expect("deserialize tauri response")
}

async fn call_result<R: for<'de> Deserialize<'de>>(
    cmd: &str,
    args: impl Serialize,
) -> Result<R, String> {
    let args = to_value(&args).expect("serialize args");
    let result = invoke(cmd, args).await;
    from_value(result).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct Empty {}

#[derive(Serialize)]
struct PathArg {
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BookPathArg {
    book_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryPathArg {
    library_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TargetFormatsArgs {
    formats: String,
    has_kepubify: bool,
    has_calibre_convert: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConvertArgs {
    book_path: String,
    target: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SendArgs {
    book_path: String,
    author: String,
}

pub async fn load_config() -> Option<Config> {
    call("load_config", Empty {}).await
}

pub async fn save_config(library_path: String) -> Result<(), String> {
    call_result("save_config", LibraryPathArg { library_path }).await
}

pub async fn validate_library(path: String) -> Result<String, String> {
    call_result("validate_library", PathArg { path }).await
}

pub async fn scan_library(path: String) -> Vec<Book> {
    call("scan_library", PathArg { path }).await
}

pub async fn parse_metadata(book_path: String) -> Option<Metadata> {
    call("parse_metadata", BookPathArg { book_path }).await
}

pub async fn available_backends() -> (bool, bool) {
    call("available_backends", Empty {}).await
}

pub async fn target_formats(
    formats: String,
    has_kepubify: bool,
    has_calibre_convert: bool,
) -> Vec<(String, String)> {
    call(
        "target_formats",
        TargetFormatsArgs {
            formats,
            has_kepubify,
            has_calibre_convert,
        },
    )
    .await
}

pub async fn convert_book(book_path: String, target: String) -> Result<ConvertOk, String> {
    call_result("convert_book", ConvertArgs { book_path, target }).await
}

pub async fn send_to_device(book_path: String, author: String) -> Result<String, String> {
    call_result("send_to_device", SendArgs { book_path, author }).await
}

pub async fn current_device() -> Option<Device> {
    call("current_device", Empty {}).await
}

#[derive(Deserialize)]
struct EventPayload {
    payload: DeviceEvent,
}

/// Subscribe to `device-event` from the Tauri backend.
/// The closure is leaked into the JS environment for the lifetime of the app —
/// acceptable here since the listener should run as long as the window is open.
pub fn listen_device_events<F>(mut on_event: F)
where
    F: FnMut(DeviceEvent) + 'static,
{
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |js: JsValue| {
        if let Ok(EventPayload { payload }) = from_value::<EventPayload>(js) {
            on_event(payload);
        }
    });
    let function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    closure.forget();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = listen_event("device-event", &function).await;
    });
}
