/// Executor agnostic task spawning
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::channel::oneshot;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

pub type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Trait abstracting over an executor.
pub trait Executor: Send + Sync {
    /// Blocks until the future has finished.
    fn block_on(&self, future: BoxedFuture);

    /// Spawns an asynchronous task using the underlying executor.
    fn spawn(&self, future: BoxedFuture) -> BoxedFuture;

    /// Runs the provided closure on a thread, which can execute blocking tasks asynchronously.
    fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send>) -> BoxedFuture;

    /// Spawns a future that doesn't implement [Send].
    ///
    /// The spawned future will be executed on the same thread that called `spawn_local`.
    ///
    /// [Send]: https://doc.rust-lang.org/std/marker/trait.Send.html
    fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) -> BoxedFuture;
}

static EXECUTOR: OnceCell<Box<dyn Executor>> = OnceCell::new();

/// Error returned by `try_register_executor` indicating that an executor was registered.
#[derive(Debug)]
pub struct ExecutorRegistered;

impl core::fmt::Display for ExecutorRegistered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "async_spawner: executor already registered")
    }
}

impl std::error::Error for ExecutorRegistered {}

pub fn try_register_executor(executor: Box<dyn Executor>) -> Result<(), ExecutorRegistered> {
    EXECUTOR.set(executor).map_err(|_| ExecutorRegistered)
}

pub fn register_executor(executor: Box<dyn Executor>) {
    try_register_executor(executor).unwrap();
}

pub fn executor() -> &'static Box<dyn Executor> {
    EXECUTOR
        .get()
        .expect("async_spawner: no executor registered")
}

/// Blocks until the future has finished.
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let lock = Arc::new(Mutex::new(None));
    let lock2 = lock.clone();
    executor().block_on(Box::pin(async move {
        let res = future.await;
        let mut lock = lock2.lock().unwrap();
        *lock = Some(res);
    }));
    let mut res = lock.lock().unwrap();
    res.take().unwrap()
}

pub struct JoinHandle<T> {
    handle: BoxedFuture,
    rx: oneshot::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Poll::Ready(()) = Pin::new(&mut self.handle).poll(cx) {
            if let Poll::Ready(Ok(res)) = Pin::new(&mut self.rx).poll(cx) {
                Poll::Ready(res)
            } else {
                panic!("task paniced");
            }
        } else {
            Poll::Pending
        }
    }
}

/// Spawns an asynchronous task using the underlying executor.
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let handle = executor().spawn(Box::pin(async move {
        let res = future.await;
        tx.send(res).ok();
    }));
    JoinHandle { handle, rx }
}

/// Runs the provided closure on a thread, which can execute blocking tasks asynchronously.
pub fn spawn_blocking<F, T>(task: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let handle = executor().spawn_blocking(Box::new(move || {
        let res = task();
        tx.send(res).ok();
    }));
    JoinHandle { handle, rx }
}

/// Spawns a future that doesn't implement [Send].
///
/// The spawned future will be executed on the same thread that called `spawn_local`.
///
/// [Send]: https://doc.rust-lang.org/std/marker/trait.Send.html
pub fn spawn_local<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let handle = executor().spawn_local(Box::pin(async move {
        let res = future.await;
        tx.send(res).ok();
    }));
    JoinHandle { handle, rx }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    #[ignore]
    async fn test_async_std() {
        struct AsyncStd;

        impl Executor for AsyncStd {
            fn block_on(&self, future: BoxedFuture) {
                async_std::task::block_on(future);
            }

            fn spawn(&self, future: BoxedFuture) -> BoxedFuture {
                Box::pin(async_std::task::spawn(future))
            }

            fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send>) -> BoxedFuture {
                Box::pin(async_std::task::spawn_blocking(task))
            }

            fn spawn_local(
                &self,
                future: Pin<Box<dyn Future<Output = ()> + 'static>>,
            ) -> BoxedFuture {
                Box::pin(async_std::task::spawn_local(future))
            }
        }

        try_register_executor(Box::new(AsyncStd)).ok();
        let res = spawn(async {
            println!("spaw on async-std");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = spawn_blocking(|| {
            println!("spawn_blocking on async-std");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = spawn_local(async {
            println!("spaw_local on async-std");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = block_on(async {
            println!("block_on on async_std");
            1
        });
        assert_eq!(res, 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_tokio() {
        struct Tokio;

        impl Executor for Tokio {
            fn block_on(&self, future: BoxedFuture) {
                tokio::runtime::Builder::new_multi_thread()
                    .build()
                    .unwrap()
                    .block_on(future);
            }

            fn spawn(&self, future: BoxedFuture) -> BoxedFuture {
                Box::pin(async { tokio::task::spawn(future).await.unwrap() })
            }

            fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send>) -> BoxedFuture {
                Box::pin(async { tokio::task::spawn_blocking(task).await.unwrap() })
            }

            fn spawn_local(
                &self,
                future: Pin<Box<dyn Future<Output = ()> + 'static>>,
            ) -> BoxedFuture {
                let handle = tokio::task::spawn_local(future);
                Box::pin(async { handle.await.unwrap() })
            }
        }

        try_register_executor(Box::new(Tokio)).ok();
        let res = spawn(async {
            println!("spaw on tokio");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = spawn_blocking(|| {
            println!("spawn_blocking on tokio");
            1
        })
        .await;
        assert_eq!(res, 1);
        tokio::task::LocalSet::new()
            .run_until(async {
                let res = spawn_local(async {
                    println!("spaw_local on tokio");
                    1
                })
                .await;
                assert_eq!(res, 1);
            })
            .await;
        spawn_blocking(|| {
            let res = block_on(async {
                println!("block_on on tokio");
                1
            });
            assert_eq!(res, 1);
        })
        .await;
    }
}
