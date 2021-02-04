
use core::{future::Future};
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "smol_executor")]
use std::sync::Arc;

enum JoinHandleInner<T> {
    #[cfg(feature = "tokio_executor")]
    Tokio(tokio::task::JoinHandle<T>),
    #[cfg(feature = "async_std_executor")]
    AsyncStd(async_std::task::JoinHandle<T>),
    #[cfg(feature = "smol_executor")]
    Smol(async_executor::Task<T>),
    #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
    RemoteHandle(futures::future::RemoteHandle<T>)
    // TODO Provide a dummy entry when no other features are enabled to use the type T, this will be disabled in any real use case
}

/// TODO Doc
pub struct JoinHandle<T> {
    inner: JoinHandleInner<T>
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = &mut self.get_mut().inner;
        match &mut inner {
            #[cfg(feature = "tokio_executor")]
            JoinHandleInner::<T>::Tokio(handle) => {
                match futures::ready!(Pin::new(handle).poll(cx)) {
                    Ok(res) => Poll::Ready(res),
                    Err(_) => core::panic!()
                }
            },
            #[cfg(feature = "async_std_executor")]
            JoinHandleInner::<T>::AsyncStd(handle) => Pin::new(handle).poll(cx),
            #[cfg(feature = "smol_executor")]
            JoinHandleInner::<T>::Smol(handle) => Pin::new(handle).poll(cx),
            #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
            JoinHandleInner::<T>::RemoteHandle(handle) =>  Pin::new(handle).poll(cx),
        }
    }
}

enum ExecutorInner {
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

use ExecutorInner::*;

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