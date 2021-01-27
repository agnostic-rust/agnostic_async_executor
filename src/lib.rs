//! Agnostic Async Executor
//! TODO Doc

#![deny(missing_docs)]
#![deny(warnings)]

use core::{future::Future};
use core::pin::Pin;
use core::task::{Context, Poll};

// TODO Get our own macros for main, test, benchmark, ... or recommend using the upstream ones
// TODO Provide Executor Agnostic time features

/// TODO Doc
pub enum JoinHandle<T> {
    /// TODO Doc
    #[cfg(feature = "tokio_executor")]
    Tokio(tokio::task::JoinHandle<T>),
    /// TODO Doc
    #[cfg(feature = "async_std_executor")]
    AsyncStd(async_std::task::JoinHandle<T>),
    /// TODO Doc
    #[cfg(feature = "smol_executor")]
    Smol(smol::Task<T>),
    /// TODO Doc
    #[cfg(feature = "wasm_bindgen_executor")]
    RemoteHandle(futures::future::RemoteHandle<T>)
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.get_mut() {
            #[cfg(feature = "tokio_executor")]
            JoinHandle::<T>::Tokio(handle) => {
                match futures::ready!(Pin::new(handle).poll(cx)) {
                    Ok(res) => Poll::Ready(res),
                    Err(_) => core::panic!()
                }
            },
            #[cfg(feature = "async_std_executor")]
            JoinHandle::<T>::AsyncStd(handle) => Pin::new(handle).poll(cx),
            #[cfg(feature = "smol_executor")]
            JoinHandle::<T>::Smol(handle) => Pin::new(handle).poll(cx),
            #[cfg(feature = "wasm_bindgen_executor")]
            JoinHandle::<T>::RemoteHandle(handle) =>  Pin::new(handle).poll(cx),
        }
    }
}

/// TODO Doc
#[derive(Debug, Clone, Copy)]
pub enum ExecutorType {
    /// TODO Doc
    #[cfg(feature = "tokio_executor")]
    Tokio,
    /// TODO Doc
    #[cfg(feature = "async_std_executor")]
    AsyncStd,
    /// TODO Doc
    #[cfg(feature = "smol_executor")]
    Smol,
    /// TODO Doc
    #[cfg(feature = "wasm_bindgen_executor")]
    WasmBindgen
}

/// TODO Doc
#[derive(Debug, Clone, Copy)]
pub struct AgnosticExecutor {
    // TODO add extra config like ideal number of thread (1, N, auto, ...)
    // TODO At least tokio needs special config to run time functions 
    executor: ExecutorType
}

impl AgnosticExecutor {

    /// TODO Doc
    pub fn new(executor: ExecutorType) -> AgnosticExecutor {
        AgnosticExecutor {
            executor
        }
    }

    /// Start the executor, if you need special configuration just start the concrete executor and don't use this function
    pub fn start<F>(&self, future: F) -> () where F: Future<Output = ()> + Send + 'static {
        match self.executor {
            #[cfg(feature = "tokio_executor")]
            ExecutorType::Tokio => {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Error creating tokio runtime")
                    .block_on(future);
            },
            #[cfg(feature = "async_std_executor")]
            ExecutorType::AsyncStd => {
                async_std::task::block_on(future);
            },
            #[cfg(feature = "smol_executor")]
            ExecutorType::Smol => {
                smol::block_on(future);
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            ExecutorType::WasmBindgen => {
                wasm_bindgen_futures::spawn_local(future);
            }
        }
    }

    /// Spawns an asynchronous task using the underlying executor.
    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        match self.executor {
            #[cfg(feature = "tokio_executor")]
            ExecutorType::Tokio => {
                JoinHandle::<T>::Tokio(tokio::task::spawn(future))
            },
            #[cfg(feature = "async_std_executor")]
            ExecutorType::AsyncStd => {
                JoinHandle::<T>::AsyncStd(async_std::task::spawn(future))
            },
            #[cfg(feature = "smol_executor")]
            ExecutorType::Smol => {
                JoinHandle::<T>::Smol(smol::spawn(future))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            ExecutorType::WasmBindgen => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandle::<T>::RemoteHandle(handle)
            }
        }
    }

    /// Runs the provided closure, and when possible, it does it in a way that doesn't block concurrent tasks.
    pub fn spawn_blocking<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        match self.executor {
            #[cfg(feature = "tokio_executor")]
            ExecutorType::Tokio => {
                JoinHandle::<T>::Tokio(tokio::task::spawn_blocking(task))
            },
            #[cfg(feature = "async_std_executor")]
            ExecutorType::AsyncStd => {
                JoinHandle::<T>::AsyncStd(async_std::task::spawn_blocking(task))
            },
            #[cfg(feature = "smol_executor")]
            ExecutorType::Smol => {
                JoinHandle::<T>::Smol(smol::spawn(smol::unblock( || task() )))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            ExecutorType::WasmBindgen => {
                use futures::future::FutureExt;
                let (future, handle) = (async { task() }).remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandle::<T>::RemoteHandle(handle)
            }
        }
    }

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
        match self.executor {
            #[cfg(feature = "tokio_executor")]
            ExecutorType::Tokio => {
                // Tokio cannot spawn_local in all relevant cases (inside other spawns for example), using a futures LocalPool instead
                let mut local = futures::executor::LocalPool::new();
                let res = local.run_until(future);
                JoinHandle::<T>::Tokio(tokio::task::spawn(async {res}))
            },
            #[cfg(feature = "async_std_executor")]
            ExecutorType::AsyncStd => {
                JoinHandle::<T>::AsyncStd(async_std::task::spawn_local(future))
            },
            #[cfg(feature = "smol_executor")]
            ExecutorType::Smol => {
                let ex = smol::LocalExecutor::new(); 
                let task = ex.spawn(future);
                let res = smol::block_on(ex.run(task));
                JoinHandle::<T>::Smol(smol::spawn(async { res }))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            ExecutorType::WasmBindgen => {
                use futures::future::FutureExt;
                let (future, handle) = future.remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandle::<T>::RemoteHandle(handle)
            }
        }
    }
    
}

/// TODO Doc
#[cfg(feature = "tokio_executor")]
pub fn tokio_executor() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::Tokio)
}

/// TODO Doc
#[cfg(feature = "async_std_executor")]
pub fn async_std_executor() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::AsyncStd)
}

/// TODO Doc
#[cfg(feature = "smol_executor")]
pub fn smol_executor() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::Smol)
}

/// TODO Doc
#[cfg(feature = "wasm_bindgen_executor")]
pub fn wasm_bindgen_executor() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::WasmBindgen)
}
