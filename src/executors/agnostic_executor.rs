use core::future::Future;

use super::join_handle::*;

#[cfg(feature = "smol_executor")]
use std::sync::Arc;

pub(crate) enum ExecutorInner {
    #[cfg(feature = "tokio_executor")]
    TokioRuntime(tokio::runtime::Runtime),
    #[cfg(feature = "async_std_executor")]
    AsyncStdRuntime,
    #[cfg(feature = "smol_executor")]
    SmolRuntime(Arc<async_executor::Executor<'static>>, usize),
    #[cfg(feature = "futures_executor")]
    FuturesRuntime(futures::executor::ThreadPool),
    #[cfg(feature = "wasm_bindgen_executor")]
    WasmBindgenRuntime
}

#[derive(Debug, Clone)]
pub(crate) enum ExecutorInnerHandle {
    #[cfg(feature = "tokio_executor")]
    TokioHandle(tokio::runtime::Handle),
    #[cfg(feature = "async_std_executor")]
    AsyncStdHandle,
    #[cfg(feature = "smol_executor")]
    SmolHandle(Arc<async_executor::Executor<'static>>),
    #[cfg(feature = "futures_executor")]
    FuturesHandle(futures::executor::ThreadPool),
    #[cfg(feature = "wasm_bindgen_executor")]
    WasmBindgenHandle
}

use ExecutorInnerHandle::*;

/// An executor that can spawn futures.
/// This can be freely stored anywhere you need, cloned, and be sent to other threads.
#[derive(Debug, Clone)]
pub struct AgnosticExecutor {
    pub(crate) inner: ExecutorInnerHandle
}

impl AgnosticExecutor {
    /// Spawns a future on this executor.
    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let inner = match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(handle) => {
                JoinHandleInner::<T>::Tokio(Some(handle.spawn(future)))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                JoinHandleInner::<T>::AsyncStd(Some(async_std::task::spawn(future)))
            }, 
            #[cfg(feature = "smol_executor")]
            SmolHandle(executor) => {
                JoinHandleInner::<T>::Smol(Some(executor.spawn(future)))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(executor) => {
                // TODO See if we can use spawn_with_handle, but maybe not a good idea if we need to introduce extra dependencies/features: https://docs.rs/futures/0.3.18/futures/task/trait.SpawnExt.html
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                executor.spawn_ok(future);
                JoinHandleInner::<T>::RemoteHandle(Some(handle))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandleInner::<T>::RemoteHandle(Some(handle))
            }
        };

        JoinHandle{inner}
    }

    /// Runs the provided closure, and when possible, it does it in a way that doesn't block concurrent tasks.
    pub fn spawn_blocking<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let inner = match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(handle) => {
                JoinHandleInner::<T>::Tokio(Some(handle.spawn_blocking(task)))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                JoinHandleInner::<T>::AsyncStd(Some(async_std::task::spawn_blocking(task)))
            },
            #[cfg(feature = "smol_executor")]
            SmolHandle(executor) => {
                JoinHandleInner::<T>::Smol(Some(executor.spawn(blocking::unblock( || task() ))))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(executor) => {
                use futures::future::FutureExt;
                let (future, handle) = (async { task()}).remote_handle();
                executor.spawn_ok(future); // TODO Maybe use blocking::unblock to make it use a threadpool instead of blocking the main one
                JoinHandleInner::<T>::RemoteHandle(Some(handle))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                use futures::future::FutureExt;
                let (future, handle) = (async { task() }).remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandleInner::<T>::RemoteHandle(Some(handle))
            }
        };

        JoinHandle{inner}
    }

    // TODO spawn_local on supported platforms

    /// Runs and blocks until completion on this executor.
    /// This function shouldn't be called from inside an async call, use await instead. In some executors it might work, but at least in tokio it doesn't.
    /// This function shouldn't be called from inside an async call, use await instead. In some executors it might work, but at least in tokio it doesn't.
    /// To be agnostic on the supported platforms use the block_on feature. If you enable it in WASM it will panic at runtime.
    #[cfg(feature = "block_on")]
    pub fn block_on<F: Future>(&self, future: F) -> F::Output
    {
        match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(handle) => {
                handle.block_on(future)
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                async_std::task::block_on(future)
            }, 
            #[cfg(feature = "smol_executor")]
            SmolHandle(_) => {
                futures_lite::future::block_on(future)
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(_) => {
                futures::executor::block_on(future)
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                unimplemented!(); // Not supported on WASM https://github.com/rustwasm/wasm-bindgen/issues/2111
            }
        }
    }
}