use wasm_bindgen::prelude::*;
//use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(inline_js = r#"
export function wasm_now() {
    return performance.now();
}"#)]
extern "C" {
    pub fn wasm_now() -> f64;
}


#[wasm_bindgen(inline_js = r#"
export async function js_delay(delay) {
    await new Promise((resolve) => {
        setTimeout(resolve, delay);
    });
}"#)]
extern "C" {
    async fn js_delay(delay: f64);
}

pub(crate) async fn wasm_timeout(duration: std::time::Duration) {
     js_delay(duration.as_secs_f64() * 1000.0).await;
}