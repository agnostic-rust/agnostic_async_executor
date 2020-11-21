/// Executor agnostic task spawning
use core::future::Future;
use core::pin::Pin;
use parking_lot::{const_rwlock, RwLock};

type Spawner = fn(Pin<Box<dyn Future<Output = ()> + Send>>);

static SPAWNER: RwLock<Option<Spawner>> = const_rwlock(None);

/// Register a spawner.
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// #[async_std::main]
/// async fn main() {
///     fn async_std_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
///         async_std::task::spawn(future);
///     }
///     async_spawn::register_spawner(async_std_spawn);
///     async_spawn::spawn(async {
///         println!("spawned on async-std");
///     });
/// }
/// ```
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// #[tokio::main]
/// async fn main() {
///     fn tokio_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
///         tokio::spawn(future);
///     }
///     async_spawn::register_spawner(tokio_spawn);
///     async_spawn::spawn(async {
///         println!("spawned on tokio");
///     });
/// }
/// ```
pub fn register_spawner(spawner: Spawner) {
    let mut lock = SPAWNER.write();
    if lock.is_some() {
        panic!("async_spawn: spawner already registered");
    }
    *lock = Some(spawner);
}

/// Spawn a task.
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// #[async_std::main]
/// async fn main() {
///     fn async_std_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
///         async_std::task::spawn(future);
///     }
///     async_spawn::register_spawner(async_std_spawn);
///     async_spawn::spawn(async {
///         println!("spawned on async-std");
///     });
/// }
/// ```
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// #[tokio::main]
/// async fn main() {
///     fn tokio_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
///         tokio::spawn(future);
///     }
///     async_spawn::register_spawner(tokio_spawn);
///     async_spawn::spawn(async {
///         println!("spawned on tokio");
///     });
/// }
/// ```
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    SPAWNER.read().expect("async_spawn: no spawner registered")(Box::pin(future));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_async_std() {
        fn async_std_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
            async_std::task::spawn(future);
        }
        register_spawner(async_std_spawn);
        spawn(async {
            println!("spawned on async-std");
        });
    }

    #[tokio::test]
    async fn test_tokio() {
        fn tokio_spawn(future: Pin<Box<dyn Future<Output = ()> + Send>>) {
            tokio::spawn(future);
        }
        register_spawner(tokio_spawn);
        spawn(async {
            println!("spawned on tokio");
        });
    }
}
