//! Tokio executor implementation.
use crate::{BoxedFuture, Executor, ExecutorRegistered};
use core::future::Future;
use core::pin::Pin;

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

    fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) -> BoxedFuture {
        let handle = tokio::task::spawn_local(future);
        Box::pin(async { handle.await.unwrap() })
    }
}

/// Try registering `tokio`.
pub fn try_register_executor() -> Result<(), ExecutorRegistered> {
    crate::try_register_executor(Box::new(Tokio))
}

/// Register `tokio`.
pub fn register_executor() {
    try_register_executor().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_tokio() {
        try_register_executor().ok();
        let res = crate::spawn(async {
            println!("spaw on tokio");
            1
        })
        .await;
        assert_eq!(res, 1);
        let res = crate::spawn_blocking(|| {
            println!("spawn_blocking on tokio");
            1
        })
        .await;
        assert_eq!(res, 1);
        tokio::task::LocalSet::new()
            .run_until(async {
                let res = crate::spawn_local(async {
                    println!("spaw_local on tokio");
                    1
                })
                .await;
                assert_eq!(res, 1);
            })
            .await;
        crate::spawn_blocking(|| {
            let res = crate::block_on(async {
                println!("block_on on tokio");
                1
            });
            assert_eq!(res, 1);
        })
        .await;
    }
}
