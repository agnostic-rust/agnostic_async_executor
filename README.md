# async-spawner

Executor independent task spawner.

```rust
use core::future::Future;
use core::pin::Pin;
type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[async_std::main]
async fn main() {
    struct AsyncStd;
    impl async_spawner::Executor for AsyncStd {
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

    async_spawner::register_executor(Box::new(AsyncStd));
    let res = async_spawner::spawn(async {
        println!("executor agnostic spawning");
        1
    })
    .await;
    assert_eq!(res, 1);
}
```

```rust
use core::future::Future;
use core::pin::Pin;
type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[tokio::main]
async fn main() {
    struct Tokio;
    impl async_spawner::Executor for Tokio {
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

    async_spawner::register_executor(Box::new(Tokio));
    let res = async_spawner::spawn(async {
        println!("executor agnostic spawning");
        1
    })
    .await;
    assert_eq!(res, 1);
}
```

# License
This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or https://opensource.org/licenses/MIT)

at your option.
