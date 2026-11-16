//! Thin bridge between the Rust simulation client and the browser host.
//!
//! The JavaScript side (src/host.js) owns everything that touches React and the
//! DOM: it builds the `update(oldState, {...})` patches that the old `js!` blocks
//! used to spell out inline. Rust only ever passes plain data across, which
//! keeps the FFI surface small and keeps JS types out of the simulation code.

use js_sys::{Array, Function, Object, Reflect};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no browser window")
}

fn host_function(name: &str) -> Function {
    let host = Reflect::get(&window(), &JsValue::from_str("cbHost"))
        .expect("window.cbHost is missing - is src/host.js loaded?");
    Reflect::get(&host, &JsValue::from_str(name))
        .expect(&format!("window.cbHost.{} is missing", name))
        .dyn_into()
        .expect(&format!("window.cbHost.{} is not a function", name))
}

/// Call a function on the JS host with the given arguments.
pub fn call(name: &str, args: &[JsValue]) -> JsValue {
    let arguments = Array::new();
    for arg in args {
        arguments.push(arg);
    }
    host_function(name)
        .apply(&JsValue::NULL, &arguments)
        .unwrap_or(JsValue::UNDEFINED)
}

pub fn call0(name: &str) -> JsValue {
    call(name, &[])
}

pub fn console_log(message: &str) {
    let _ = call("log", &[JsValue::from_str(message)]);
}

pub fn console_error(message: &str) {
    let _ = call("error", &[JsValue::from_str(message)]);
}

/// Build a plain JS object from key/value pairs (one level).
pub fn object(pairs: Vec<(&str, JsValue)>) -> JsValue {
    let obj = Object::new();
    for (key, value) in pairs {
        let _ = Reflect::set(&obj, &JsValue::from_str(key), &value);
    }
    obj.into()
}

pub fn object_set(target: &JsValue, key: &str, value: JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(key), &value);
}

pub fn empty_object() -> JsValue {
    Object::new().into()
}

pub fn array(values: Vec<JsValue>) -> JsValue {
    let arr = Array::new();
    for value in values {
        arr.push(&value);
    }
    arr.into()
}

pub fn f32_array(values: &[f32]) -> JsValue {
    js_sys::Float32Array::from(values).into()
}

pub fn u16_array(values: &[u16]) -> JsValue {
    js_sys::Uint16Array::from(values).into()
}

/// Serialize like the old `stdweb::serde::Serde` did: JSON-compatible values
/// (plain objects with string keys, arrays), never JS `Map`s.
pub fn to_js<T: Serialize>(value: &T) -> JsValue {
    if let Ok(converted) = value.serialize(&serde_wasm_bindgen::Serializer::json_compatible()) {
        return converted;
    }

    // The json-compatible serializer rejects maps with non-string keys (e.g.
    // `HashMap<MachineID, isize>`), which stdweb's serde_json-based conversion
    // used to stringify instead. Fall back to that behaviour rather than
    // silently sending null.
    match serde_json::to_string(value) {
        Ok(json) => js_sys::JSON::parse(&json).unwrap_or(JsValue::NULL),
        Err(error) => {
            console_error(&format!("Could not serialize value for JS: {}", error));
            JsValue::NULL
        }
    }
}

pub fn from_js<T: DeserializeOwned>(value: JsValue) -> Option<T> {
    serde_wasm_bindgen::from_value(value).ok()
}

/// Read `window.<path...>`, returning `undefined` if any step is missing.
pub fn get_path(path: &[&str]) -> JsValue {
    let mut current: JsValue = window().into();
    for key in path {
        match Reflect::get(&current, &JsValue::from_str(key)) {
            Ok(next) => current = next,
            Err(_) => return JsValue::UNDEFINED,
        }
    }
    current
}

pub fn set_global(name: &str, value: JsValue) {
    let _ = Reflect::set(&window().into(), &JsValue::from_str(name), &value);
}
