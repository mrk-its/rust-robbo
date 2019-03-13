extern crate web_sys;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub fn log(text: &str) {
    console::log_1(&JsValue::from_str(text));
}

macro_rules! log {
    ( $( $x:expr ),* ) => {log(&format!($( $x, )*))}
}
