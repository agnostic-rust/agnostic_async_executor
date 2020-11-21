# async-spawn

Executor independent task spawner.

```rust
use core::future::Future;
use core::pin::Pin;

#[async_std::main]
async fn main() {
    fn async_std_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        async_std::task::spawn(future);
    }
    async_spawn::register_spawner(async_std_spawn);
    async_spawn::spawn(async {
        println!("spawned on async-std");
    });
}
```

```rust
use core::future::Future;
use core::pin::Pin;

#[tokio::main]
async fn main() {
    fn tokio_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        tokio::spawn(future);
    }
    async_spawn::register_spawner(tokio_spawn);
    async_spawn::spawn(async {
        println!("spawned on tokio");
    });
}
```

# License
This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or https://opensource.org/licenses/MIT)

at your option.
