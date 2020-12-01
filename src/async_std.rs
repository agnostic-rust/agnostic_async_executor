//! Async std executor implementation.
use crate::{BoxedFuture, Executor, ExecutorRegistered};
use core::future::Future;
use core::pin::Pin;

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

    fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) -> BoxedFuture {
        Box::pin(async_std::task::spawn_local(future))
    }
}

/// Try registering `async-std`.
pub fn try_register_executor() -> Result<(), ExecutorRegistered> {
    crate::try_register_executor(Box::new(AsyncStd))
}

/// Register `async-std`.
pub fn register_executor() {
    try_register_executor().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    #[ignore]
    async fn test_async_std() {
        try_register_executor().ok();
        let res = crate::spawn(async {
            println!("spaw on async-std");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = crate::spawn_blocking(|| {
            println!("spawn_blocking on async-std");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = crate::spawn_local(async {
            println!("spaw_local on async-std");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = crate::block_on(async {
            println!("block_on on async_std");
            1
        });
        assert_eq!(res, 1);
    }
}
