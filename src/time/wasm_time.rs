use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function wasm_now() {
    return performance.now();
}"#)]
extern "C" {
    pub fn wasm_now() -> f64;
}

#[wasm_bindgen(inline_js = r#"
export function wasm_log(msg) {
    return console.log(msg);
}"#)]
extern "C" {
    pub fn wasm_log(msg: &str);
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

pub(crate) struct WasmInterval {
    delay: f64,
    next_interval: f64,
}

impl WasmInterval {
    pub(crate) fn new(duration: std::time::Duration) -> Self {
        let delay  = duration.as_secs_f64() * 1000.0;
        let next_interval = wasm_now() + delay;
        WasmInterval {delay, next_interval}
    }

    pub(crate) async fn next(&mut self) {
        let remaining = self.next_interval - wasm_now();
        //wasm_log(&format!("Next: {:.2} Remaining: {:.2} Delay: {:.2}", self.next_interval,  remaining, self.delay));
        js_delay(remaining).await;
        self.next_interval += self.delay;
    }
}