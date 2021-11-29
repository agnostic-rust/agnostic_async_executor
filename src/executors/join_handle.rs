use core::{
    pin::Pin,
    task::{Context, Poll},
    future::Future
};

pub(crate) enum JoinHandleInner<T> {
    #[cfg(feature = "tokio_executor")]
    Tokio(Option<tokio::task::JoinHandle<T>>),
    #[cfg(feature = "async_std_executor")]
    AsyncStd(Option<async_std::task::JoinHandle<T>>),
    #[cfg(feature = "smol_executor")]
    Smol(Option<async_executor::Task<T>>),
    #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
    RemoteHandle(Option<futures::future::RemoteHandle<T>>)
}

impl<T> JoinHandleInner<T> {
    async fn cancel(mut self) {
        match &mut self {
            #[cfg(feature = "tokio_executor")]
            JoinHandleInner::<T>::Tokio(handle) => { 
                if let Some(handle) = handle.take() {
                    handle.abort();
                }
            },
            #[cfg(feature = "async_std_executor")]
            JoinHandleInner::<T>::AsyncStd(handle) => { 
                if let Some(handle) = handle.take() {
                    handle.cancel().await;
                }
            },
            #[cfg(feature = "smol_executor")]
            JoinHandleInner::<T>::Smol(handle) => { drop(handle.take()) },
            #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
            JoinHandleInner::<T>::RemoteHandle(handle) =>  { drop(handle.take()) },
        }
        
    }
}

impl<T> Drop for JoinHandleInner<T> {
    fn drop(&mut self) {
        match self {
            #[cfg(feature = "tokio_executor")]
            JoinHandleInner::<T>::Tokio(handle) => {},
            #[cfg(feature = "async_std_executor")]
            JoinHandleInner::<T>::AsyncStd(handle) => {},
            #[cfg(feature = "smol_executor")]
            JoinHandleInner::<T>::Smol(handle) => {
                if let Some(handle) = handle.take() {
                    handle.detach(); // We need to detach to avoid canceling the task if we drop the handle
                }
            },
            #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
            JoinHandleInner::<T>::RemoteHandle(handle) =>  {
                if let Some(handle) = handle.take() {
                    handle.forget(); // We need to forget to avoid canceling the task if we drop the handle
                }
            },
        }
        
    }
}

/// A future holding the result of a spawned async task
pub struct JoinHandle<T> {
    pub(crate) inner: JoinHandleInner<T>
}

impl<T> JoinHandle<T> {
    /// Cancels the task associated with the handle. You need to await this function for the cancellation to occur.
    pub async fn cancel(self) {
        self.inner.cancel().await;
    }
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = T;

    // TODO Think if we need something better than unwrap on the handle. 
    // In theory cancel consumes the future, and is the only way to have a None there.

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = &mut self.get_mut().inner;
        match &mut inner {
            #[cfg(feature = "tokio_executor")]
            JoinHandleInner::<T>::Tokio(handle) => {
                match futures::ready!(Pin::new(handle.as_mut().unwrap()).poll(cx)) {
                    Ok(res) => Poll::Ready(res),
                    Err(e) => panic!("Tokio JoinHandle error: {}", e)
                }
            },
            #[cfg(feature = "async_std_executor")]
            JoinHandleInner::<T>::AsyncStd(handle) => Pin::new(handle.as_mut().unwrap()).poll(cx),
            #[cfg(feature = "smol_executor")]
            JoinHandleInner::<T>::Smol(handle) => Pin::new(handle.as_mut().unwrap()).poll(cx),
            #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
            JoinHandleInner::<T>::RemoteHandle(handle) =>  Pin::new(handle.as_mut().unwrap()).poll(cx),
        }
    }
}

