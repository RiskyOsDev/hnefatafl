use wasm_bindgen::prelude::*;
use web_sys::{js_sys::Function, window};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let w = window().expect("can't get window, probably not running in a browser");
    let d: web_sys::HtmlDocument = { 
        let temp_d = w.document().unwrap();
        temp_d.dyn_into::<web_sys::HtmlDocument>()?
    };
    
    let start_b: web_sys::HtmlElement = d.create_element("button")?.dyn_into()?;
    start_b.set_onclick(Some(&Function::new_no_args("start_connect()")));
    start_b.set_text_content(Some("start connection"));
    d.body().unwrap().append_child(&start_b)?;
    
    let connect_b: web_sys::HtmlElement = d.create_element("button")?.dyn_into()?;
    connect_b.set_onclick(Some(&Function::new_no_args("respond_connect()")));
    connect_b.set_text_content(Some("respond to connection"));
    d.body().unwrap().append_child(&connect_b)?;

    Ok(())
}

#[wasm_bindgen]
pub fn start_connect() {
    log("start connecting");
}

#[wasm_bindgen]
pub fn respond_connect() {
    log("respond to connection");
}
