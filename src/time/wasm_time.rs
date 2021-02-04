use std::{pin::Pin, task::{Context, Poll}};

use std::future::Future;
use futures::FutureExt;
use js_sys::Promise;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::TimedOut;

#[wasm_bindgen(inline_js = r#"
export function js_now() {
    return performance.now();
}"#)]
extern "C" {
    pub fn js_now() -> f64;
}

pub(crate) fn wasm_now() -> u64 {
    js_now() as u64
}

#[wasm_bindgen(inline_js = r#"
export function js_delay(delay) {
    return new Promise((resolve) => {
        setTimeout(resolve, delay);
    });
}"#)]
extern "C" {
    fn js_delay(delay: f64) -> Promise;
}

pub(crate) struct WasmSleepFuture {
    inner: SendWrapper<JsFuture> // This is need to be send compatible even if wasm is single threaded
}

impl WasmSleepFuture {
    pub fn new(duration: std::time::Duration) -> Self {
        WasmSleepFuture::new_millis(duration.as_secs_f64() as u64 * 1000)
    }

    pub fn new_millis(duration: u64) -> Self {
        let delay_promise = js_delay(duration as f64);
        let delay_future = JsFuture::from(delay_promise);
        WasmSleepFuture {
            inner: SendWrapper::new(delay_future)
        }
    }
}

impl Future for WasmSleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match futures::ready!(Pin::new(&mut (*self.inner)).poll(cx)) {
            Ok(_) => Poll::Ready(()),
            Err(_) => panic!()
        }
    }
}
pub(crate) struct WasmTimeoutFuture<F> where F: Future + Unpin {
    future: F,
    timeout: WasmSleepFuture
}

impl<F> WasmTimeoutFuture<F> where F: Future + Unpin {
    pub fn new(future: F, duration: std::time::Duration) -> Self {
        WasmTimeoutFuture{ 
            future: future,
            timeout: WasmSleepFuture::new(duration)
        }
    }

    pub fn new_millis(future: F, duration: u64) -> Self {
        WasmTimeoutFuture{ 
            future: future,
            timeout: WasmSleepFuture::new_millis(duration)
        }
    }
}

impl<F> Future for WasmTimeoutFuture<F> where F: Future + Unpin {
    type Output = Result<F::Output, super::TimedOut>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Poll::Ready(t) = self.future.poll_unpin(cx) {
            return Poll::Ready(Ok(t));
        }

        if let Poll::Ready(_) = self.timeout.poll_unpin(cx) {
            return Poll::Ready(Err(TimedOut));
        }

        Poll::Pending
    }
}


pub(crate) struct WasmInterval {
    delay: u64,
    next_interval: u64,
}

impl WasmInterval {
    pub(crate) fn new(duration: std::time::Duration) -> Self {
        let delay  = (duration.as_secs_f64() * 1000.0) as u64;
        let next_interval = wasm_now() + delay;
        WasmInterval {delay, next_interval}
    }

    pub(crate) async fn next(&mut self) {
        let remaining = self.next_interval - wasm_now();
        WasmSleepFuture::new_millis(remaining).await;
        self.next_interval += self.delay;
    }
}