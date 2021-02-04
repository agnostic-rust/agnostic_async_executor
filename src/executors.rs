
use core::{future::Future};


#[cfg(feature = "smol_executor")]
use std::sync::Arc;

mod join_handle;
pub use join_handle::*;

mod agnostic_executor;
pub use agnostic_executor::*;

use ExecutorInner::*;
use ExecutorInnerHandle::*;

/// TODO Doc
pub struct AgnosticExecutorBuilder {}

impl AgnosticExecutorBuilder {

    /// TODO Doc
    #[cfg(feature = "tokio_executor")]
    pub fn use_tokio_executor(self) -> AgnosticExecutorManager {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Error creating tokio runtime");
        let handle = rt.handle().clone();
        AgnosticExecutorManager { 
            inner_handle: TokioHandle(handle),
            inner_runtime: TokioRuntime(rt)
        }
    }

    /// TODO Doc
    #[cfg(feature = "tokio_executor")]
    pub fn use_tokio_executor_with_runtime(self, rt: tokio::runtime::Runtime) -> AgnosticExecutorManager {
        let handle = rt.handle().clone();
        AgnosticExecutorManager { 
            inner_handle: TokioHandle(handle),
            inner_runtime: TokioRuntime(rt)
        }
    }

    /// TODO Doc
    #[cfg(feature = "async_std_executor")]
    pub fn use_async_std_executor(self) -> AgnosticExecutorManager {
        AgnosticExecutorManager { 
            inner_handle: AsyncStdHandle,
            inner_runtime: AsyncStdRuntime
        }
    }

    /// TODO Doc If num_threads is not provided default to the number of logical cores
    #[cfg(feature = "smol_executor")]
    pub fn use_smol_executor(self, num_threads: Option<usize>) -> AgnosticExecutorManager {
        let rt = Arc::new(async_executor::Executor::new());
        let handle = rt.clone();
        let num_threads = num_threads.unwrap_or(num_cpus::get());
        AgnosticExecutorManager { 
            inner_handle: SmolHandle(handle),
            inner_runtime: SmolRuntime(rt, num_threads)
        }
    }

    /// TODO Doc
    #[cfg(feature = "futures_executor")]
    pub fn use_futures_executor(self) -> AgnosticExecutorManager {
        let rt = futures::executor::ThreadPool::new().expect("Error creating the futures threadpool");
        let handle = rt.clone();
        AgnosticExecutorManager { 
            inner_handle: FuturesHandle(handle),
            inner_runtime: FuturesRuntime(rt)
        }
    }

    /// TODO Doc
    #[cfg(feature = "futures_executor")]
    pub fn use_futures_executor_with_runtime(self, rt: futures::executor::ThreadPool) -> AgnosticExecutorManager {
        let handle = rt.clone();
        AgnosticExecutorManager { 
            inner_handle: FuturesHandle(handle),
            inner_runtime: FuturesRuntime(rt)
        }
    }

    /// TODO Doc
    #[cfg(feature = "wasm_bindgen_executor")]
    pub fn use_wasm_bindgen_executor(self) -> AgnosticExecutorManager {
        AgnosticExecutorManager { 
            inner_handle: WasmBindgenHandle,
            inner_runtime: WasmBindgenRuntime
        }
    }
}

use once_cell::sync::OnceCell;
static GLOBAL_EXECUTOR: OnceCell<AgnosticExecutor>  = OnceCell::new();

/// TODO Doc
pub struct AgnosticExecutorManager {
    inner_runtime: ExecutorInner,
    inner_handle: ExecutorInnerHandle
}

impl AgnosticExecutorManager {
    /// TODO Doc
    pub fn get_executor(&self) -> AgnosticExecutor {
        AgnosticExecutor { inner: self.inner_handle.clone() }
    }

    // /// TODO Doc
    // pub fn get_local_executor(&self) -> LocalAgnosticExecutor {
    //     LocalAgnosticExecutor { inner: self.inner_handle.clone() }
    // }

    /// TODO Doc
    pub fn set_as_global(&self) {
        GLOBAL_EXECUTOR.set(self.get_executor()).expect("Global executor already set");
    }

    /// Start the executor
    pub fn start<F>(self, future: F) where F: Future<Output = ()> + Send + 'static {
        match self.inner_runtime {
            #[cfg(feature = "tokio_executor")]
            TokioRuntime(runtime) => {
                runtime.block_on(future);
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdRuntime => {
                async_std::task::block_on(future);
            },
            #[cfg(feature = "smol_executor")]
            SmolRuntime(executor, num_threads) => {
                if num_threads > 1 {
                    let (signal, shutdown) = async_channel::unbounded::<()>();
                    easy_parallel::Parallel::new()
                        .each(0..num_threads, |_| futures_lite::future::block_on(executor.run(shutdown.recv())))
                        .finish(|| futures_lite::future::block_on(async {
                            future.await;
                            drop(signal);
                        }));
                } else {
                    futures_lite::future::block_on(executor.run(future));
                }
            },
            #[cfg(feature = "futures_executor")]
            FuturesRuntime(runtime) => {
                runtime.spawn_ok(future);
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenRuntime => {
                wasm_bindgen_futures::spawn_local(future);
            }
        }
    }

    /// Start the executor
    pub fn start_local<F>(self, future: F) where F: Future<Output = ()> + 'static {
        match self.inner_runtime {
            #[cfg(feature = "tokio_executor")]
            TokioRuntime(runtime) => {
                runtime.block_on(future);
            },
            #[cfg(feature = "async_std_executor")]
            AsyncStdRuntime => {
                async_std::task::block_on(future);
            },
            #[cfg(feature = "smol_executor")]
            SmolRuntime(executor, num_threads) => {
                if num_threads > 1 {
                    let (signal, shutdown) = async_channel::unbounded::<()>();
                    easy_parallel::Parallel::new()
                        .each(0..num_threads, |_| futures_lite::future::block_on(executor.run(shutdown.recv())))
                        .finish(|| futures_lite::future::block_on(async {
                            future.await;
                            drop(signal);
                        }));
                } else {
                    futures_lite::future::block_on(executor.run(future));
                }
            },
            #[cfg(feature = "futures_executor")]
            FuturesRuntime(_runtime) => {
                unimplemented!();
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            WasmBindgenRuntime => {
                wasm_bindgen_futures::spawn_local(future);
            }
        }
    }
}

/// TODO Doc
pub fn new_agnostic_executor() -> AgnosticExecutorBuilder {
    AgnosticExecutorBuilder {}
}

/// TODO Doc
pub fn get_global_executor() -> &'static AgnosticExecutor {
    GLOBAL_EXECUTOR.get().expect("No global executor set")
}

/// TODO Doc
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
{
    get_global_executor().spawn(future)
}

/// TODO Doc
pub fn spawn_blocking<F, T>(task: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    get_global_executor().spawn_blocking(task)
}

// /// TODO Doc
// pub fn spawn_local<F, T>(future: F) -> JoinHandle<T>
// where
//     F: Future<Output = T> + 'static,
//     T: Send + 'static,
// {
//     get_global_executor().spawn_local(future)
// }