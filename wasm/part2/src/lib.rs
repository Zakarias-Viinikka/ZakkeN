mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LiveForever {
    data: String,
}

#[wasm_bindgen]
impl LiveForever {
    pub fn new(initial: String) -> LiveForever {
        LiveForever { data: initial }
    }

    pub fn get_data(&self) -> String {
        self.data.clone()
    }

    //console.log(state.change_data("new data"));
    pub fn change_data(&mut self, new_data: String) {
        self.data = new_data;
    }
}

// https://wasm-bindgen.github.io/wasm-bindgen/examples/wasm-in-web-worker.html
