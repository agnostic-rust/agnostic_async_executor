//! TODO Doc

use crate::{AgnosticExecutor, BlockingError, BoxedFuture, Executor};
use core::future::Future;
use core::pin::Pin;

struct Tokio {}

impl Executor for Tokio {

    fn block_on(&self, _future: BoxedFuture) -> Result<(), BlockingError> {
        // TODO When this issue is resolved https://github.com/tokio-rs/tokio/pull/3097
        //let handle = tokio::runtime::Handle::try_current()?;
        //handle.block_on(future);
        Err(BlockingError {})
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

/// TODO Doc
pub fn tokio() -> AgnosticExecutor {
    AgnosticExecutor::new(Box::new(Tokio {}))
}

