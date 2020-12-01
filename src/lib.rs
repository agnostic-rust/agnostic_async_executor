//! Executor agnostic task spawning
//!
//! ```rust
//! #[async_std::main]
//! async fn main() {
//!     async_spawner::async_std::register_executor();
//!     let res = async_spawner::spawn(async {
//!         println!("executor agnostic spawning");
//!         1
//!     })
//!     .await;
//!     assert_eq!(res, 1);
//! }
//! ```
//!
//! ```rust
//! #[tokio::main]
//! async fn main() {
//!     async_spawner::tokio::register_executor();
//!     let res = async_spawner::spawn(async {
//!         println!("executor agnostic spawning");
//!         1
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
use futures_util::future::FutureExt;
use futures_util::stream::{Stream, StreamExt};
use once_cell::sync::OnceCell;

#[cfg(feature = "async-std")]
pub mod async_std;
#[cfg(feature = "tokio")]
pub mod tokio;

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

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

/// Tries registering an executor.
pub fn try_register_executor(executor: Box<dyn Executor>) -> Result<(), ExecutorRegistered> {
    EXECUTOR.set(executor).map_err(|_| ExecutorRegistered)
}

/// Register an executor. Panics if an executor was already registered.
pub fn register_executor(executor: Box<dyn Executor>) {
    try_register_executor(executor).unwrap();
}

/// Returns the registered executor.
pub fn executor() -> &'static dyn Executor {
    &**EXECUTOR
        .get()
        .expect("async_spawner: no executor registered")
}

/// Blocks until the future has finished.
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, mut rx) = async_channel::bounded(1);
    executor().block_on(Box::pin(async move {
        let res = future.await;
        tx.try_send(res).ok();
    }));
    rx.next().now_or_never().unwrap().unwrap()
}

/// Executor agnostic join handle.
pub struct JoinHandle<T> {
    handle: BoxedFuture,
    rx: async_channel::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Poll::Ready(()) = Pin::new(&mut self.handle).poll(cx) {
            if let Poll::Ready(Some(res)) = Pin::new(&mut self.rx).poll_next(cx) {
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
    let (tx, rx) = async_channel::bounded(1);
    let handle = executor().spawn(Box::pin(async move {
        let res = future.await;
        tx.try_send(res).ok();
    }));
    JoinHandle { handle, rx }
}

/// Runs the provided closure on a thread, which can execute blocking tasks asynchronously.
pub fn spawn_blocking<F, T>(task: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = async_channel::bounded(1);
    let handle = executor().spawn_blocking(Box::new(move || {
        let res = task();
        tx.try_send(res).ok();
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
    let (tx, rx) = async_channel::bounded(1);
    let handle = executor().spawn_local(Box::pin(async move {
        let res = future.await;
        tx.try_send(res).ok();
    }));
    JoinHandle { handle, rx }
}
