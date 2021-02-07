
use core::{future::Future};


use std::rc::Rc;
#[cfg(feature = "smol_executor")]
use std::sync::Arc;

mod join_handle;
pub use join_handle::*;

mod agnostic_executor;
pub use agnostic_executor::*;

use ExecutorInner::*;
use ExecutorInnerHandle::*;

mod local_agnostic_executor;
pub use local_agnostic_executor::*;

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
        self.use_tokio_executor_with_runtime(rt)
    }

    /// TODO Doc
    #[cfg(feature = "tokio_executor")]
    pub fn use_tokio_executor_with_runtime(self, rt: tokio::runtime::Runtime) -> AgnosticExecutorManager {
        let handle = rt.handle().clone();
        AgnosticExecutorManager { 
            inner_handle: TokioHandle(handle),
            inner_runtime: TokioRuntime(rt),
            local_inner_runtime: LocalExecutorInnerRuntime::TokioRuntime(tokio::task::LocalSet::new()),
            local_inner_handle: Some(LocalExecutorInnerHandle::TokioHandle),
            finish_callback: None
        }
    }

    /// TODO Doc
    #[cfg(feature = "async_std_executor")]
    pub fn use_async_std_executor(self) -> AgnosticExecutorManager {
        AgnosticExecutorManager { 
            inner_handle: AsyncStdHandle,
            inner_runtime: AsyncStdRuntime,
            local_inner_runtime: LocalExecutorInnerRuntime::AsyncStdRuntime,
            local_inner_handle: Some(LocalExecutorInnerHandle::AsyncStdHandle),
            finish_callback: None
        }
    }

    /// TODO Doc If num_threads is not provided default to the number of logical cores
    #[cfg(feature = "smol_executor")]
    pub fn use_smol_executor(self, num_threads: Option<usize>) -> AgnosticExecutorManager {
        let rt = Arc::new(async_executor::Executor::new());
        let handle = rt.clone();
        let num_threads = num_threads.unwrap_or(num_cpus::get());
        let local = Rc::new(async_executor::LocalExecutor::new());
        AgnosticExecutorManager { 
            inner_handle: SmolHandle(handle),
            inner_runtime: SmolRuntime(rt, num_threads),
            local_inner_runtime: LocalExecutorInnerRuntime::SmolRuntime(local.clone()),
            local_inner_handle: Some(LocalExecutorInnerHandle::SmolHandle(local)),
            finish_callback: None
        }
    }

    /// TODO Doc
    #[cfg(feature = "futures_executor")]
    pub fn use_futures_executor(self) -> AgnosticExecutorManager {
        let rt = futures::executor::ThreadPool::new().expect("Error creating the futures threadpool");
        self.use_futures_executor_with_runtime(rt)
    }

    /// TODO Doc
    #[cfg(feature = "futures_executor")]
    pub fn use_futures_executor_with_runtime(self, rt: futures::executor::ThreadPool) -> AgnosticExecutorManager {
        let handle = rt.clone();
        let local = futures::executor::LocalPool::new();
        let local_spawner = local.spawner();
        AgnosticExecutorManager { 
            inner_handle: FuturesHandle(handle),
            inner_runtime: FuturesRuntime(rt),
            local_inner_runtime: LocalExecutorInnerRuntime::FuturesRuntime(local),
            local_inner_handle: Some(LocalExecutorInnerHandle::FuturesHandle(local_spawner)),
            finish_callback: None
        }
    }

    /// TODO Doc
    #[cfg(feature = "wasm_bindgen_executor")]
    pub fn use_wasm_bindgen_executor(self) -> AgnosticExecutorManager {
        AgnosticExecutorManager { 
            inner_handle: WasmBindgenHandle,
            inner_runtime: WasmBindgenRuntime,
            local_inner_runtime: LocalExecutorInnerRuntime::WasmBindgenRuntime,
            local_inner_handle: Some(LocalExecutorInnerHandle::WasmBindgenHandle),
            finish_callback: None
        }
    }
}

use once_cell::sync::OnceCell;
static GLOBAL_EXECUTOR: OnceCell<AgnosticExecutor>  = OnceCell::new();

/// TODO Doc
pub struct AgnosticExecutorManager {
    inner_runtime: ExecutorInner,
    inner_handle: ExecutorInnerHandle,
    local_inner_runtime: LocalExecutorInnerRuntime,
    local_inner_handle: Option<LocalExecutorInnerHandle>,
    finish_callback: Option<Box<dyn FnOnce() -> () + 'static>>
}

impl AgnosticExecutorManager {
    /// TODO Doc
    pub fn get_executor(&self) -> AgnosticExecutor {
        AgnosticExecutor { inner: self.inner_handle.clone() }
    }

    // TODO See if we can make local executor implement clone

    /// TODO Doc Can only be called once because it doesn't implement clone
    pub fn get_local_executor(&mut self) -> LocalAgnosticExecutor {
        LocalAgnosticExecutor { inner: self.local_inner_handle.take().expect("You cannot get the loca executor twice") }
    }

    /// Sets up a callback to be called when the executor finishes. It can only be called once.
    /// This is needed because the start() call might finish before it's future completes and we might need a way to detect completion.
    pub fn on_finish<F>(&mut self, cb: F) where F: FnOnce() -> () + 'static {
        self.finish_callback = Some(Box::new(cb));
    }

    /// TODO Doc
    pub fn set_as_global(&self) {
        GLOBAL_EXECUTOR.set(self.get_executor()).expect("Global executor already set");
    }

    /// Start the executor.
    /// In wasm the call might finish before the future has completely executed due to the non-blocking nature of the environment, so don't depend on this.
    /// With the other executors you can depend on the fact that start is blocking, but the on_finish callback is called anyway.
    pub fn start<F>(mut self, future: F) where F: Future<Output = ()> + 'static {
        let cb = self.finish_callback.take();
        let finish_cb = || {
            if let Some(cb) = cb {
                cb();
            }
        };

        match (self.inner_runtime, self.local_inner_runtime) {
            #[cfg(feature = "tokio_executor")]
            (TokioRuntime(runtime), LocalExecutorInnerRuntime::TokioRuntime(localset)) => {
                runtime.block_on(async move {
                    localset.run_until(future).await;
                });
                finish_cb();
            },
            #[cfg(feature = "async_std_executor")]
            (AsyncStdRuntime, _) => {
                async_std::task::block_on(future);
                finish_cb();
            },
            #[cfg(feature = "smol_executor")]
            (SmolRuntime(executor, num_threads),  LocalExecutorInnerRuntime::SmolRuntime(local)) => {
                if num_threads > 1 {
                    let (signal, shutdown) = async_channel::unbounded::<()>();
                    easy_parallel::Parallel::new()
                        .each(0..num_threads, |_| futures_lite::future::block_on(executor.run(shutdown.recv())))
                        .finish(|| {
                            futures_lite::future::block_on(async {
                                local.run(future).await;
                                drop(signal);
                            });
                            finish_cb();
                        });
                } else {
                    futures_lite::future::block_on(local.run(future));
                    finish_cb();
                }
            },
            #[cfg(feature = "futures_executor")]
            (FuturesRuntime(_), LocalExecutorInnerRuntime::FuturesRuntime(mut local)) => {
                local.run_until(future);
                finish_cb();
            },
            #[cfg(feature = "wasm_bindgen_executor")]
            (WasmBindgenRuntime, _) => {
                wasm_bindgen_futures::spawn_local(async move {
                    future.await;
                    finish_cb();
                });
            },
            _ => { panic!("Couldn't start the local executor, maybe you forgot to call get_local_executor before starting."); }
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