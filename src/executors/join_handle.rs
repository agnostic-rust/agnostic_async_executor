use core::{
    pin::Pin,
    task::{Context, Poll},
    future::Future
};

pub(crate) enum JoinHandleInner<T> {
    #[cfg(feature = "tokio_executor")]
    Tokio(tokio::task::JoinHandle<T>),
    #[cfg(feature = "async_std_executor")]
    AsyncStd(async_std::task::JoinHandle<T>),
    #[cfg(feature = "smol_executor")]
    Smol(async_executor::Task<T>),
    #[cfg(any(feature = "wasm_bindgen_executor", feature = "futures_executor"))]
    RemoteHandle(futures::future::RemoteHandle<T>)
}

/// A future holding the result of a spawned async task
pub struct JoinHandle<T> {
    pub(crate) inner: JoinHandleInner<T>
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
                    Err(e) => panic!("Tokio JoinHandle error: {}", e)
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

