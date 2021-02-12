
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


/// It lets you build an AgnosticExecutorManager for a concrete executor
pub struct AgnosticExecutorBuilder {}

impl AgnosticExecutorBuilder {

    /// A manager for a default multi-threaded Tokio executor
    #[cfg(feature = "tokio_executor")]
    pub fn use_tokio_executor(self) -> AgnosticExecutorManager {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Error creating tokio runtime");
        self.use_tokio_executor_with_runtime(rt)
    }

    /// A manager for a provided Tokio executor
    #[cfg(feature = "tokio_executor")]
    pub fn use_tokio_executor_with_runtime(self, rt: tokio::runtime::Runtime) -> AgnosticExecutorManager {
        let handle = rt.handle().clone();
        AgnosticExecutorManager { 
            inner_handle: TokioHandle(handle),
            inner_runtime: TokioRuntime(rt),
            local_inner_runtime: LocalExecutorInnerRuntime::TokioRuntime(tokio::task::LocalSet::new()),
            local_inner_handle: LocalExecutorInnerHandle::TokioHandle,
            finish_callback: None
        }
    }

    /// A manager for an Async Std executor
    #[cfg(feature = "async_std_executor")]
    pub fn use_async_std_executor(self) -> AgnosticExecutorManager {
        AgnosticExecutorManager { 
            inner_handle: AsyncStdHandle,
            inner_runtime: AsyncStdRuntime,
            local_inner_runtime: LocalExecutorInnerRuntime::AsyncStdRuntime,
            local_inner_handle: LocalExecutorInnerHandle::AsyncStdHandle,
            finish_callback: None
        }
    }

    /// A manager for a Smol executor.
    /// If num_threads is not provided, it defaults default to the number of logical cores.
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
            local_inner_handle: LocalExecutorInnerHandle::SmolHandle(local),
            finish_callback: None
        }
    }

    /// A manager for a default Threadpool executor from the futures crate.
    #[cfg(feature = "futures_executor")]
    pub fn use_futures_executor(self) -> AgnosticExecutorManager {
        let rt = futures::executor::ThreadPool::new().expect("Error creating the futures threadpool");
        self.use_futures_executor_with_runtime(rt)
    }

    /// A manager for a provided executor from the futures crate.
    #[cfg(feature = "futures_executor")]
    pub fn use_futures_executor_with_runtime(self, rt: futures::executor::ThreadPool) -> AgnosticExecutorManager {
        let handle = rt.clone();
        let local = futures::executor::LocalPool::new();
        let local_spawner = local.spawner();
        AgnosticExecutorManager { 
            inner_handle: FuturesHandle(handle),
            inner_runtime: FuturesRuntime(rt),
            local_inner_runtime: LocalExecutorInnerRuntime::FuturesRuntime(local),
            local_inner_handle: LocalExecutorInnerHandle::FuturesHandle(local_spawner),
            finish_callback: None
        }
    }

    /// A manager for a wasm executor from the wasm_bindgen_futures crate
    #[cfg(feature = "wasm_bindgen_executor")]
    pub fn use_wasm_bindgen_executor(self) -> AgnosticExecutorManager {
        AgnosticExecutorManager { 
            inner_handle: WasmBindgenHandle,
            inner_runtime: WasmBindgenRuntime,
            local_inner_runtime: LocalExecutorInnerRuntime::WasmBindgenRuntime,
            local_inner_handle: LocalExecutorInnerHandle::WasmBindgenHandle,
            finish_callback: None
        }
    }
}

use once_cell::sync::OnceCell;
static GLOBAL_EXECUTOR: OnceCell<AgnosticExecutor>  = OnceCell::new();

/// An AgnosticExecutorManager is configured on creation for a specific executor and it allows you get the the general and local executor, set it as global executor, and of course, start the executor.
pub struct AgnosticExecutorManager {
    inner_runtime: ExecutorInner,
    inner_handle: ExecutorInnerHandle,
    local_inner_runtime: LocalExecutorInnerRuntime,
    local_inner_handle: LocalExecutorInnerHandle,
    finish_callback: Option<Box<dyn FnOnce() -> () + 'static>>
}

impl AgnosticExecutorManager {
    /// Get the executor of this manager as an AgnosticExecutor.
    /// This is needed if you need to spawn new tasks, and it be easily stored, cloned and send across threads to have it available where ever you need to spawn a new tasks or interact with the executor. 
    pub fn get_executor(&self) -> AgnosticExecutor {
        AgnosticExecutor { inner: self.inner_handle.clone() }
    }

    /// Get the local executor of this manager as a LocalAgnosticExecutor.
    /// A local executor is similar to the general executor but it allows to spawn tasks that are not send.
    /// The drawback is that, even tough you can store and clone a LocalAgnosticExecutor, you cannot send it to other threads.
    pub fn get_local_executor(&mut self) -> LocalAgnosticExecutor {
        LocalAgnosticExecutor { inner: self.local_inner_handle.clone() }
    }

    /// Sets up a callback to be called when the executor finishes. It can only be called once.
    /// This is needed because on wasm the start() call might finish before it's future completes and we might need a way to detect completion.
    pub fn on_finish<F>(&mut self, cb: F) where F: FnOnce() -> () + 'static {
        self.finish_callback = Some(Box::new(cb));
    }

    /// Sets this executor as the global executor to be used with the global crate functions get_global_executor, spawn and spawn_blocking.
    /// You still need to start the executor after setting it as global.
    /// This can only be called once.
    pub fn set_as_global(&self) {
        GLOBAL_EXECUTOR.set(self.get_executor()).expect("Global executor already set");
    }

    /// Start the executor with the provided future. 
    /// This future doesn't need to be Send, but it needs to be 'static. You can use async move {...} to achieve this if needed.
    /// Note that in wasm the call might finish before the future has completely executed due to the non-blocking nature of the environment, so don't depend on this.
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
            _ => { panic!("Couldn't start the executor."); }
        }
    }
}

/// The base function to create a new concrete AgnosticExecutor.
/// Use the builder to specify the concrete executor, and then the manager to get access the executor and start it.
pub fn new_agnostic_executor() -> AgnosticExecutorBuilder {
    AgnosticExecutorBuilder {}
}

/// Gets the executor set as the global executor.
/// It might panic if no executor is set.
pub fn get_global_executor() -> &'static AgnosticExecutor {
    GLOBAL_EXECUTOR.get().expect("No global executor set")
}

/// Spawn a future on the global executor
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
{
    get_global_executor().spawn(future)
}

/// Runs the provided closure on the global executor, and when possible, it does it in a way that doesn't block concurrent tasks.
pub fn spawn_blocking<F, T>(task: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    get_global_executor().spawn_blocking(task)
}