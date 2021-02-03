#![ cfg(feature = "time") ]

use std::time::Duration;

use crate::AgnosticExecutor;
use crate::agnostic_executor::ExecutorInnerHandle::*;

#[cfg(feature = "wasm_bindgen_executor")]
mod wasm_time;

#[cfg(feature = "wasm_bindgen_executor")]
pub use wasm_time::*;

/// TODO Doc
#[derive(Debug, Clone, Copy)]
pub struct TimedOut;
enum IntervalInner {
    #[cfg(feature = "tokio_executor")]
    Tokio(tokio::time::Interval),
    #[cfg(feature = "async_std_executor")]
    AsyncStd(async_std::stream::Interval),
    #[cfg(feature = "smol_executor")]
    Smol(std::cell::Cell<async_io::Timer>, Duration, std::time::Instant),
    #[cfg(feature = "futures_executor")]
    AsyncTimer(async_timer::Interval),
    #[cfg(feature = "wasm_bindgen_executor")]
    WasmBindgen(WasmInterval)
}

/// TODO Doc
pub struct Interval(IntervalInner);

impl Interval {
    /// TODO Doc
    pub async fn next(&mut self) {
        match &mut self.0 {
            #[cfg(feature = "tokio_executor")]
            IntervalInner::Tokio(interval) => {
                interval.tick().await;
            },
            #[cfg(feature = "async_std_executor")]
            IntervalInner::AsyncStd(interval) => {
                use futures_lite::StreamExt;
                interval.next().await;
            },
            #[cfg(feature = "smol_executor")]
            IntervalInner::Smol(timer, duration, at) => {
                use std::ops::Add;
                timer.get_mut().await;
                *at = at.add(*duration);
                timer.set(async_io::Timer::at(*at));
            },
            #[cfg(feature = "futures_executor")]
            IntervalInner::AsyncTimer(interval) => {
                interval.wait().await;
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            IntervalInner::WasmBindgen(wasm_interval) => {
                wasm_interval.next().await;
            }
        }
    }
}

// TODO Implement Stream for interval when is in std: https://github.com/rust-lang/rust/issues/79024

impl AgnosticExecutor {

    /// TODO Doc
    pub async fn sleep(&self, duration: Duration) {
        match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(_) => {
                tokio::time::sleep(duration).await;
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                async_std::task::sleep(duration).await;
            }, 
            #[cfg(feature = "smol_executor")]
            SmolHandle(_) => {
                async_io::Timer::after(duration).await;
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(_) => {
                async_timer::new_timer(duration).await;
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                wasm_time::wasm_timeout(duration).await;
            }
        }        
    }

    /// TODO Doc
    pub async fn timeout<T: futures::Future>(&self, duration: Duration, future: T) -> Result<T::Output, TimedOut> {
        match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(_) => {
                tokio::time::timeout(duration, future).await.map_err(|_| TimedOut)
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                async_std::future::timeout(duration, future).await.map_err(|_| TimedOut)
            }, 
            #[cfg(feature = "smol_executor")]
            SmolHandle(_) => {
                futures_lite::future::or(async {
                    async_io::Timer::after(duration).await; Err(TimedOut)
                }, async {
                    Ok(future.await)
                }).await
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(_) => {
                futures::pin_mut!(future);
                async_timer::timed(future, duration).await.map_err(|_| TimedOut)
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                // TODO Handle this implementing a custom future in wasm_time to avoid select_biased! and the dependency
                use futures::{FutureExt, select_biased};
                let mut future = Box::pin(future.fuse());
                let mut delay = Box::pin(wasm_time::wasm_timeout(duration).fuse());
                select_biased! {
                    res = future => Ok(res),
                    _ = delay => Err(TimedOut),
                }
            }
        }        
    }

    /// TODO Doc
    pub fn interval(&self, duration: Duration) -> Interval {
        match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(_) => {
                use std::ops::Add;
                let at = tokio::time::Instant::now().add(duration);
                // This is needed because by default tokio intervals fire immediately
                Interval(IntervalInner::Tokio(tokio::time::interval_at(at, duration)))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                Interval(IntervalInner::AsyncStd(async_std::stream::interval(duration)))
            }, 
            #[cfg(feature = "smol_executor")]
            SmolHandle(_) => {
                use std::ops::Add;
                let at = std::time::Instant::now().add(duration);
                let timer = std::cell::Cell::new(async_io::Timer::at(at));
                Interval(IntervalInner::Smol(timer, duration, at))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(_) => {
                Interval(IntervalInner::AsyncTimer(async_timer::interval(duration)))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                Interval(IntervalInner::WasmBindgen(WasmInterval::new(duration)))
            }
        }        
    }

}