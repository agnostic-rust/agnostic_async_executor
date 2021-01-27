//! Agnostic Async Executor
//!
//! ```rust
//! #[tokio::main(flavor = "multi_thread")]
//! async fn main() {
//!     let exec = agnostic_async_executor::tokio();
//!     let res = exec.spawn(async {
//!         println!("executor agnostic spawning");
//!         1i32
//!     })
//!     .await;
//!     assert_eq!(res, 1);
//! }
//! ```
//!
//! ```rust
//! #[async_std::main]
//! async fn main() {
//!     let exec = agnostic_async_executor::async_std();
//!     let res = exec.spawn(async {
//!         println!("executor agnostic spawning");
//!         1i32
//!     })
//!     .await;
//!     assert_eq!(res, 1);
//! }
//! ```

#![deny(missing_docs)]
#![deny(warnings)]

use core::{future::Future};
use core::pin::Pin;
use core::task::{Context, Poll};

// TODO Get our own macros for main, test, benchmark, ... then find a way for test setup and share the testing functions among all executors
// TODO block_on should only be used to start a new executor, never inside a task, probably is better to provide a generic one that works even in tokio right now. For special configuration just use the upstream block_on

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
    RemoteHandle(futures_util::future::RemoteHandle<T>)
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.get_mut() {
            #[cfg(feature = "tokio_executor")]
            JoinHandle::<T>::Tokio(handle) => {
                match futures_util::ready!(Pin::new(handle).poll(cx)) {
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
#[derive(Debug)]
pub struct BlockingError;

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
    executor: ExecutorType
}

impl AgnosticExecutor {
    /// TODO Doc
    pub fn new(executor: ExecutorType) -> AgnosticExecutor {
        AgnosticExecutor {
            executor
        }
    }

    
    /// Blocks until the future has finished.
    pub fn block_on<F, T>(&self, future: F) -> Result<T, BlockingError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        match self.executor {
            #[cfg(feature = "tokio_executor")]
            ExecutorType::Tokio => {
                Err(BlockingError) // TODO Implement when this issue is fixed https://github.com/tokio-rs/tokio/pull/3097
            },
            #[cfg(feature = "async_std_executor")]
            ExecutorType::AsyncStd => {
                Ok(async_std::task::block_on(future))
            },
            #[cfg(feature = "smol_executor")]
            ExecutorType::Smol => {
                Ok(smol::block_on(future))
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            ExecutorType::WasmBindgen => {
                drop(future);
                Err(BlockingError)
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
                use futures_util::FutureExt;
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
                use futures_util::FutureExt;
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

        // TODO We should either integrate tokio::task::LocalSet::new().run_until inside this call or remove smol::block_on for consistency

        match self.executor {
            #[cfg(feature = "tokio_executor")]
            ExecutorType::Tokio => {
                JoinHandle::<T>::Tokio(tokio::task::spawn_local(future))
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
                use futures_util::FutureExt;
                let (future, handle) = future.remote_handle();
                wasm_bindgen_futures::spawn_local(future);
                JoinHandle::<T>::RemoteHandle(handle)
            }
        }
    }
    
}

/// TODO Doc
#[cfg(feature = "tokio_executor")]
pub fn tokio() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::Tokio)
}

/// TODO Doc
#[cfg(feature = "async_std_executor")]
pub fn async_std() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::AsyncStd)
}

/// TODO Doc
#[cfg(feature = "smol_executor")]
pub fn smol() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::Smol)
}

/// TODO Doc
#[cfg(feature = "wasm_bindgen_executor")]
pub fn wasm_bindgen() -> AgnosticExecutor {
    AgnosticExecutor::new(ExecutorType::WasmBindgen)
}