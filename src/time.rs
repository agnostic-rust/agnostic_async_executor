//! A set of time utilities useful when working with async code that work with every executor, including wasm.
//! When possible, they are implemented using the executor provided primitives to make it as performance and accurate as possible.

#![ cfg(feature = "time") ]

use std::time::Duration;

use crate::AgnosticExecutor;
use crate::executors::ExecutorInnerHandle::*;

#[cfg(feature = "wasm_bindgen_executor")]
mod wasm_time;

#[cfg(feature = "wasm_bindgen_executor")]
pub use wasm_time::*;


fn to_millis(duration: Duration) -> u64 {
    (duration.as_secs_f64() * 1000.0) as u64
}

/// Error returned when a future times out.
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

/// An interval be used to retrieve a sequence of futures, each one expiring after a given interval from the previous one.
pub struct Interval(IntervalInner);

impl Interval {
    /// Returns the future for the next interval in the sequence.
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

impl AgnosticExecutor {

    /// Returns a future that sleeps for a duration.
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
                WasmSleepFuture::new(duration).await;
            }
        }        
    }

    /// Returns a future that sleeps for a duration in milliseconds.
    pub async fn sleep_millis(&self, duration: u64) {
        self.sleep(Duration::from_millis(duration)).await;
    }

    /// Wraps a future in a timeout that expires after a duration if the provided future didn't finish.
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
                futures::pin_mut!(future);
                WasmTimeoutFuture::new(future, duration).await
            }
        }        
    }

    /// Wraps a future in a timeout that expires after a duration in milliseconds if the provided future didn't finish.
    pub async fn timeout_millis<T: futures::Future>(&self, duration: u64, future: T) -> Result<T::Output, TimedOut> {
        self.timeout(Duration::from_millis(duration), future).await
    }
    
    /// Creates a new Interval for a given duration.
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

    /// Creates a new Interval for a given duration in milliseconds.
    pub fn interval_millis(&self, duration: u64) -> Interval {
        self.interval(Duration::from_millis(duration))
    }

}

/// A Stopwatch can be used to measure time in a generic way that works even on wasm.
pub struct Stopwatch {
    #[cfg(not(feature = "wasm_bindgen_executor"))]
    start: std::time::Instant,
    #[cfg(feature = "wasm_bindgen_executor")]
    start: u64,
    tolerance: u64
}

impl Stopwatch {
    /// Creates a new Stopwatch instance. You can have multiple independent instances.
    pub fn new() -> Self {
        Stopwatch::new_tolerant_millis(0)
    }

    /// Creates a new Stopwatch instance with a given tolerance for the has_elapsed* checks.
    pub fn new_tolerant(tolerance: Duration) -> Self {
        Stopwatch::new_tolerant_millis(to_millis(tolerance))
    }

    /// Creates a new Stopwatch instance with a given tolerance in milliseconds for the has_elapsed* checks.
    pub fn new_tolerant_millis(tolerance: u64) -> Self {
        Stopwatch {
            #[cfg(not(feature = "wasm_bindgen_executor"))]
            start: std::time::Instant::now(),
            #[cfg(feature = "wasm_bindgen_executor")]
            start: wasm_now() as u64,
            tolerance
        }  
    }

    /// Sets the tolerance for the has_elapsed* checks.
    pub fn set_tolerance(&mut self, tolerance: Duration) {
        self.tolerance = to_millis(tolerance);
    }

    /// Sets the tolerance in milliseconds for the has_elapsed* checks.
    pub fn set_tolerance_millis(&mut self, tolerance: u64) {
        self.tolerance = tolerance;
    }

    /// Returns the duration since creation or last reset.
    pub fn elapsed(&self) -> Duration {
        #[cfg(not(feature = "wasm_bindgen_executor"))]
        return self.start.elapsed();
        #[cfg(feature = "wasm_bindgen_executor")]
        return Duration::from_millis(wasm_now() - self.start);
    }

    /// Returns the duration in milliseconds since creation or last reset.
    pub fn elapsed_millis(&self) -> u64 {
        #[cfg(not(feature = "wasm_bindgen_executor"))]
        return to_millis(self.start.elapsed());
        #[cfg(feature = "wasm_bindgen_executor")]
        return wasm_now() - self.start;
    }

    /// Checks if a duration has elapsed since creation or last reset, with a given tolerance configured on this instance.
    pub fn has_elapsed(&self, duration: Duration) -> bool {
        self.has_elapsed_millis(to_millis(duration))
    }

    /// Checks if a duration in milliseconds has elapsed since creation or last reset, with a given tolerance configured on this instance.
    pub fn has_elapsed_millis(&self, duration: u64) -> bool {
        self.elapsed_millis() + self.tolerance >= duration
    }

    /// Resets this instance so that the elapsed time is measured from this instant.
    pub fn reset(&mut self) {
        #[cfg(not(feature = "wasm_bindgen_executor"))]
        { self.start = std::time::Instant::now(); }
        #[cfg(feature = "wasm_bindgen_executor")]
        { self.start = wasm_now() as u64; }
    }
}