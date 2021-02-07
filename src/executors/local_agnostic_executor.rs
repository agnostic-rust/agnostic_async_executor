use core::future::Future;
use std::rc::Rc;

use super::join_handle::*;

pub(crate) enum LocalExecutorInnerRuntime {
    #[cfg(feature = "tokio_executor")]
    TokioRuntime(tokio::task::LocalSet),
    #[cfg(feature = "async_std_executor")]
    AsyncStdRuntime,
    #[cfg(feature = "smol_executor")]
    SmolRuntime(Rc<async_executor::LocalExecutor<'static>>),
    #[cfg(feature = "futures_executor")]
    FuturesRuntime(futures::executor::LocalPool),
    #[cfg(feature = "wasm_bindgen_executor")]
    WasmBindgenRuntime
}

#[derive(Debug)]
pub(crate) enum LocalExecutorInnerHandle {
    #[cfg(feature = "tokio_executor")]
    TokioHandle,
    #[cfg(feature = "async_std_executor")]
    AsyncStdHandle,
    #[cfg(feature = "smol_executor")]
    SmolHandle(Rc<async_executor::LocalExecutor<'static>>),
    #[cfg(feature = "futures_executor")]
    FuturesHandle(futures::executor::LocalSpawner),
    #[cfg(feature = "wasm_bindgen_executor")]
    WasmBindgenHandle
}

use LocalExecutorInnerHandle::*;
use futures::task::LocalSpawnExt;

/// TODO Doc
#[derive(Debug)]
pub struct LocalAgnosticExecutor {
    pub(crate) inner: LocalExecutorInnerHandle
}

impl LocalAgnosticExecutor {

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
        let inner = match &self.inner {
            #[cfg(feature = "tokio_executor")]
            TokioHandle => {
                JoinHandleInner::<T>::Tokio(tokio::task::spawn_local(future))
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdHandle => {
                JoinHandleInner::<T>::AsyncStd(async_std::task::spawn_local(future))
            },
            #[cfg(feature = "smol_executor")]
            SmolHandle(executor) => {
                JoinHandleInner::<T>::Smol(executor.spawn(future))
            },
            #[cfg(feature = "futures_executor")]
            FuturesHandle(executor) => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                executor.spawn_local(future).expect("Local spawn error");
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
    
}