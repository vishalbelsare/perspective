/******************************************************************************
 *
 * Copyright (c) 2019, the Perspective Authors.
 *
 * This file is part of the Perspective library, distributed under the terms of
 * the Apache License 2.0.  The full license can be found in the LICENSE file.
 *
 */
mod api;
mod arrow;
mod utils;

use wasm_bindgen::prelude::*;

use crate::arrow::ArrowAccessor;
use crate::api::load_arrow_stream;
use crate::utils::set_panic_hook;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn load_arrow(buffer: Box<[u8]>) -> *const ArrowAccessor {
    set_panic_hook();
    let accessor = load_arrow_stream(buffer);
    Box::into_raw(accessor)
}

#[wasm_bindgen]
pub fn accessor_pprint(accessor: *const ArrowAccessor) {
    unsafe {
        log(format!("{}", accessor.as_ref().unwrap()).as_str())
    }
}

#[wasm_bindgen]
pub fn accessor_get(_accessor: *const ArrowAccessor, column_name: &str, ridx: usize) -> JsValue {
    log(format!("[Rust] called get() for {}[{}]", column_name, ridx).as_str());
    JsValue::NULL
}