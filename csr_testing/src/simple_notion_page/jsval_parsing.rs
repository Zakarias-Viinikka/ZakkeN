use anyhow::{Result, anyhow};
use wasm_bindgen::{JsCast, JsValue}; // let array: js_sys::Array = js_value

//I tried putting mod js_value_parsing in main.rs in final_example/
use js_sys::Array;

pub fn js_value_to_usize_tuple(js_value: JsValue) -> Result<(usize, usize)> {
    //code expects an array from js
    let array = js_value
        .dyn_into::<Array>()
        .map_err(|_| anyhow!("expected an array from JsValue"))?;

    if array.length() == 2 {
        let first = js_value_to_usize(array.get(0))?;
        let second = js_value_to_usize(array.get(1))?;

        Ok((first, second))
    } else {
        return Err(anyhow!("expected array with 2 items"));
    }
}

fn js_value_to_usize(js_value: JsValue) -> Result<usize> {
    js_value
        .as_string()
        .ok_or(anyhow!("not a string"))?
        .parse()
        .map_err(|e| anyhow!("parse error: {}", e))
}
