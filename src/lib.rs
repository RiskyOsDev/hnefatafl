use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    alert("hello");

    Ok(())
}
