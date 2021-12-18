use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn foo() {
    web_sys::console::log_1(&JsValue::from("Hello from Rust!"))
}
