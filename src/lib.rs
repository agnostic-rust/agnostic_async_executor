//! Agnostic Async Executor
//!
//! ```rust
//! #[tokio::main(flavor = "multi_thread")]
//! async fn main() {
//!     let exec = agnostic_async_executor::tokio::tokio();
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
//!     let exec = agnostic_async_executor::async_std::async_std();
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

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_channel::oneshot;
use futures_util::future::FutureExt;

#[cfg(feature = "async-std")]
pub mod async_std;
#[cfg(feature = "tokio")]
pub mod tokio;

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;


/// TODO Doc
#[derive(Debug)]
pub struct BlockingError {}

/// Trait abstracting over an executor.
pub trait Executor: Send + Sync {
    /// Blocks until the future has finished.
    fn block_on(&self, future: BoxedFuture) -> Result<(), BlockingError>;

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

/// TODO Doc
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
                panic!("Future panic!");
            }
        } else {
            Poll::Pending
        }
    }
}

/// TODO Doc
pub struct AgnosticExecutor {
    executor: Box<dyn Executor>
}

impl AgnosticExecutor {
    /// TODO Doc
    pub fn new(executor: Box<dyn Executor>) -> AgnosticExecutor {
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
        let (tx, rx) = oneshot::channel();
        self.executor.block_on(Box::pin(async move {
            let res = future.await;
            tx.send(res).ok();
        }))?;
        Ok(rx.now_or_never().unwrap().unwrap())
    }

    /// Spawns an asynchronous task using the underlying executor.
    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let handle = self.executor.spawn(Box::pin(async move {
            let res = future.await;
            tx.send(res).ok();
        }));
        JoinHandle { handle, rx }
    }

    /// Runs the provided closure on a thread, which can execute blocking tasks asynchronously.
    pub fn spawn_blocking<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let handle = self.executor.spawn_blocking(Box::new(move || {
            let res = future();
            tx.send(res).ok();
        }));
        JoinHandle { handle, rx }
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
        let (tx, rx) = oneshot::channel();
        let handle = self.executor.spawn_local(Box::pin(async move {
            let res = future.await;
            tx.send(res).ok();
        }));
        JoinHandle { handle, rx }
    }

    
}