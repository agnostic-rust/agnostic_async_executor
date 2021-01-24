//! TODO Doc

use crate::{AgnosticExecutor, BlockingError, BoxedFuture, Executor};
use core::future::Future;
use core::pin::Pin;

struct AsyncStd;

impl Executor for AsyncStd {
    fn block_on(&self, future: BoxedFuture) -> Result<(), BlockingError> {
        async_std::task::block_on(future);
        Ok(())
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

/// TODO Doc
pub fn async_std() -> AgnosticExecutor {
    AgnosticExecutor::new(Box::new(AsyncStd))
}