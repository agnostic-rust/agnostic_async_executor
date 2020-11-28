//! Executor agnostic task spawning
//!
//! ```rust
//! # #[macro_use] extern crate async_spawner;
//! # use async_spawner::{Bump, BumpBox, BumpFuture};
//! # use core::future::Future;
//! # use core::pin::Pin;
//! #[async_std::main]
//! async fn main() {
//!     struct AsyncStd;
//!     impl async_spawner::Executor for AsyncStd {
//!         fn block_on(&self, future: Pin<&mut (dyn Future<Output = ()> + Send + 'static)>) {
//!             async_std::task::block_on(future);
//!         }
//!
//!         fn spawn(&self, bump: &Bump, future: BumpFuture) -> BumpFuture {
//!             async_spawner::coerce_bump_box_pin!(
//!                 BumpBox::new_in(async_std::task::spawn(future), bump)
//!             )
//!         }
//!
//!         fn spawn_blocking(&self, bump: &Bump, task: Box<dyn FnOnce() + Send>) -> BumpFuture {
//!             async_spawner::coerce_bump_box_pin!(
//!                 BumpBox::new_in(async_std::task::spawn_blocking(task), bump)
//!             )
//!         }
//!
//!         fn spawn_local(&self, bump: &Bump, future: Pin<BumpBox<'static, dyn Future<Output = ()>>>) -> BumpFuture {
//!             async_spawner::coerce_bump_box_pin!(
//!                 BumpBox::new_in(async_std::task::spawn_local(future), bump)
//!             )
//!         }
//!     }
//!
//!     async_spawner::register_executor(Box::new(AsyncStd));
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
//! # #[macro_use] extern crate async_spawner;
//! # use async_spawner::{Bump, BumpBox, BumpFuture};
//! # use core::future::Future;
//! # use core::pin::Pin;
//! #[tokio::main]
//! async fn main() {
//!     struct Tokio;
//!     impl async_spawner::Executor for Tokio {
//!         fn block_on(&self, future: Pin<&mut (dyn Future<Output = ()> + Send + 'static)>) {
//!             tokio::runtime::Builder::new_multi_thread()
//!                 .build()
//!                 .unwrap()
//!                 .block_on(future);
//!         }
//!
//!         fn spawn(&self, bump: &Bump, future: BumpFuture) -> BumpFuture {
//!             async_spawner::coerce_bump_box_pin!(
//!                 BumpBox::new_in(async { tokio::task::spawn(future).await.unwrap() }, bump)
//!             )
//!         }
//!
//!         fn spawn_blocking(&self, bump: &Bump, task: Box<dyn FnOnce() + Send>) -> BumpFuture {
//!             async_spawner::coerce_bump_box_pin!(
//!                 BumpBox::new_in(async { tokio::task::spawn_blocking(task).await.unwrap() }, bump)
//!             )
//!         }
//!
//!         fn spawn_local(&self, bump: &Bump, future: Pin<BumpBox<'static, dyn Future<Output = ()>>>) -> BumpFuture {
//!             let handle = tokio::task::spawn_local(future);
//!             async_spawner::coerce_bump_box_pin!(
//!                 BumpBox::new_in(async { handle.await.unwrap() }, bump)
//!             )
//!         }
//!     }
//!
//!     async_spawner::register_executor(Box::new(Tokio));
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
pub use bumpalo::boxed::Box as BumpBox;
pub use bumpalo::Bump;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

/// Type alias for a bump allocated future.
pub type BumpFuture = Pin<BumpBox<'static, dyn Future<Output = ()> + Send>>;

/// Coerces a `BumpBox<'a, impl T>` to `BumpBox<'static, dyn T>`.
#[macro_export]
macro_rules! coerce_bump_box {
    ($b:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            let coerce = Box::from_raw(BumpBox::into_raw($b)) as Box<_>;
            BumpBox::from_raw(Box::into_raw(coerce))
        }
    };
}

/// Coerces and pins a `BumpBox<'a, impl T>` as a `Pin<BumpBox<'static, dyn T>>`.
#[macro_export]
macro_rules! coerce_bump_box_pin {
    ($b:expr) => {
        unsafe { Pin::new_unchecked(coerce_bump_box!($b)) }
    };
}

/// Trait abstracting over an executor.
pub trait Executor: Send + Sync {
    /// Blocks until the future has finished.
    fn block_on(&self, future: Pin<&mut (dyn Future<Output = ()> + Send + 'static)>);

    /// Spawns an asynchronous task using the underlying executor.
    fn spawn(&self, bump: &Bump, future: BumpFuture) -> BumpFuture;

    /// Runs the provided closure on a thread, which can execute blocking tasks asynchronously.
    fn spawn_blocking(&self, bump: &Bump, task: Box<dyn FnOnce() + Send>) -> BumpFuture;

    /// Spawns a future that doesn't implement [Send].
    ///
    /// The spawned future will be executed on the same thread that called `spawn_local`.
    ///
    /// [Send]: https://doc.rust-lang.org/std/marker/trait.Send.html
    fn spawn_local(
        &self,
        bump: &Bump,
        future: Pin<BumpBox<'static, dyn Future<Output = ()>>>,
    ) -> BumpFuture;
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
    let cell = Mutex::new(None);
    let cell_ref: &'static Mutex<Option<T>> = unsafe { &*(&cell as *const _) };
    let future = async move {
        let res = future.await;
        let mut cell = cell_ref.lock().unwrap();
        *cell = Some(res);
    };
    futures::pin_mut!(future);
    executor().block_on(future);
    let mut cell = cell.lock().unwrap();
    cell.take().unwrap()
}

/// Executor agnostic join handle.
pub struct JoinHandle<T: 'static> {
    handle: BumpFuture,
    res: BumpBox<'static, Mutex<Option<T>>>,
    #[allow(unused)]
    bump: Bump,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Poll::Ready(()) = Pin::new(&mut self.handle).poll(cx) {
            let mut res = self.res.lock().unwrap();
            Poll::Ready(res.take().unwrap())
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
    let bump = Bump::new();
    let bump_ref = unsafe { &*(&bump as *const _) };
    let res = BumpBox::new_in(Mutex::new(None), bump_ref);
    let res_ref: &'static Mutex<Option<T>> = unsafe { &*(&*res as *const _) };
    let future = BumpBox::new_in(
        async move {
            let res = future.await;
            let mut cell = res_ref.lock().unwrap();
            *cell = Some(res);
        },
        &bump,
    );
    let handle = executor().spawn(&bump, coerce_bump_box_pin!(future));
    JoinHandle { bump, handle, res }
}

/// Runs the provided closure on a thread, which can execute blocking tasks asynchronously.
pub fn spawn_blocking<F, T>(task: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let bump = Bump::new();
    let bump_ref = unsafe { &*(&bump as *const _) };
    let res = BumpBox::new_in(Mutex::new(None), bump_ref);
    let res_ref: &'static Mutex<Option<T>> = unsafe { &*(&*res as *const _) };
    let task = Box::new(move || {
        let res = task();
        let mut cell = res_ref.lock().unwrap();
        *cell = Some(res);
    });
    let handle = executor().spawn_blocking(&bump, task);
    JoinHandle { bump, handle, res }
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
    let bump = Bump::new();
    let bump_ref = unsafe { &*(&bump as *const _) };
    let res = BumpBox::new_in(Mutex::new(None), bump_ref);
    let res_ref: &'static Mutex<Option<T>> = unsafe { &*(&*res as *const _) };
    let future = BumpBox::new_in(
        async move {
            let res = future.await;
            let mut cell = res_ref.lock().unwrap();
            *cell = Some(res);
        },
        &bump,
    );
    let handle = executor().spawn_local(&bump, coerce_bump_box_pin!(future));
    JoinHandle { bump, handle, res }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    #[ignore]
    async fn test_async_std() {
        struct AsyncStd;

        impl Executor for AsyncStd {
            fn block_on(&self, future: Pin<&mut (dyn Future<Output = ()> + Send + 'static)>) {
                async_std::task::block_on(future);
            }

            fn spawn(&self, bump: &Bump, future: BumpFuture) -> BumpFuture {
                coerce_bump_box_pin!(BumpBox::new_in(async_std::task::spawn(future), bump))
            }

            fn spawn_blocking(&self, bump: &Bump, task: Box<dyn FnOnce() + Send>) -> BumpFuture {
                coerce_bump_box_pin!(BumpBox::new_in(async_std::task::spawn_blocking(task), bump))
            }

            fn spawn_local(
                &self,
                bump: &Bump,
                future: Pin<BumpBox<'static, dyn Future<Output = ()>>>,
            ) -> BumpFuture {
                coerce_bump_box_pin!(BumpBox::new_in(async_std::task::spawn_local(future), bump))
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
            fn block_on(&self, future: Pin<&mut (dyn Future<Output = ()> + Send + 'static)>) {
                tokio::runtime::Builder::new_multi_thread()
                    .build()
                    .unwrap()
                    .block_on(future);
            }

            fn spawn(&self, bump: &Bump, future: BumpFuture) -> BumpFuture {
                coerce_bump_box_pin!(BumpBox::new_in(
                    async move { tokio::task::spawn(future).await.unwrap() },
                    bump,
                ))
            }

            fn spawn_blocking(&self, bump: &Bump, task: Box<dyn FnOnce() + Send>) -> BumpFuture {
                coerce_bump_box_pin!(BumpBox::new_in(
                    async { tokio::task::spawn_blocking(task).await.unwrap() },
                    bump,
                ))
            }

            fn spawn_local(
                &self,
                bump: &Bump,
                future: Pin<BumpBox<'static, dyn Future<Output = ()>>>,
            ) -> BumpFuture {
                let handle = tokio::task::spawn_local(future);
                coerce_bump_box_pin!(BumpBox::new_in(async { handle.await.unwrap() }, bump))
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
