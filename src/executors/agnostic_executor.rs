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

/// TODO Doc
#[derive(Debug, Clone)]
pub struct AgnosticExecutor {
    pub(crate) inner: ExecutorInnerHandle
}

impl AgnosticExecutor {
    /// Spawns an asynchronous task using the underlying executor.
    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let inner = match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(handle) => {
                JoinHandleInner::<T>::Tokio(handle.spawn(future))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                JoinHandleInner::<T>::AsyncStd(async_std::task::spawn(future))
            }, 
            #[cfg(feature = "smol_executor")]
            SmolHandle(executor) => {
                JoinHandleInner::<T>::Smol(executor.spawn(future))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(executor) => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                executor.spawn_ok(future);
                JoinHandleInner::<T>::RemoteHandle(handle)
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandleInner::<T>::RemoteHandle(handle)
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
                JoinHandleInner::<T>::Tokio(handle.spawn_blocking(task))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                JoinHandleInner::<T>::AsyncStd(async_std::task::spawn_blocking(task))
            },
            #[cfg(feature = "smol_executor")]
            SmolHandle(executor) => {
                JoinHandleInner::<T>::Smol(executor.spawn(blocking::unblock( || task() )))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(executor) => {
                use futures::future::FutureExt;
                let (future, handle) = (async { task()}).remote_handle();
                executor.spawn_ok(future);
                JoinHandleInner::<T>::RemoteHandle(handle)
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                use futures::future::FutureExt;
                let (future, handle) = (async { task() }).remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandleInner::<T>::RemoteHandle(handle)
            }
        };

        JoinHandle{inner}
    }


    /*
    /// Spawns a future that doesn't implement [Send].
    ///
    /// The spawned future will be executed on the same thread that called `spawn_local`.
    ///
    /// [Send]: https://doc.rust-lang.org/std/marker/trait.Send.html
    pub fn spawn_local<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + 'static,
        T: Send + 'static,
    {
        // TODO Use a thread local to store the LocalPool/LocalExecutor https://doc.rust-lang.org/std/macro.thread_local.html
        // TODO Take ideas from async_global_executor that is used inside async_std and internally uses smol async_executor.
        let inner = match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle(_) => {
                // Tokio cannot spawn_local in all relevant cases (inside other spawns for example), using a futures LocalPool instead
                let mut local = futures::executor::LocalPool::new();
                let res = local.run_until(future);
                JoinHandleInner::<T>::Tokio(tokio::task::spawn(async {res}))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                JoinHandleInner::<T>::AsyncStd(async_std::task::spawn_local(future))
            },
            #[cfg(feature = "smol_executor")]
            SmolHandle(executor) => {
                let ex = async_executor::LocalExecutor::new();
                let task = ex.spawn(future);
                let res = futures_lite::future::block_on(ex.run(task));
                JoinHandleInner::<T>::Smol(executor.spawn(async { res }))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(executor) => {
                // FIXME This doesn't work !!!!
                // Maybe put spawn_local under a feature, and in this case the future executor is single threaded (LocalPool instead of ThreadPool).
                use futures::future::FutureExt;
                let mut local = futures::executor::LocalPool::new();
                let res = local.run_until(future);
                let (future, handle) = (async { res }).remote_handle();
                executor.spawn_ok(future);
                JoinHandleInner::<T>::RemoteHandle(handle)
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenHandle => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandleInner::<T>::RemoteHandle(handle)
            }
        };

        JoinHandle{inner}
    }
    */
    
}