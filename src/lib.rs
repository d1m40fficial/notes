mod app;
mod edit;
mod icons;
mod markdown;
mod model;
mod storage;
mod util;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    app::mount()
}
