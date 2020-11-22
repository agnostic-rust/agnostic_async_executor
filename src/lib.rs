/// Executor agnostic task spawning
use core::future::Future;
use core::pin::Pin;
use parking_lot::{const_rwlock, RwLock};

type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
type Spawner = fn(Boxed) -> Boxed;

static SPAWNER: RwLock<Option<Spawner>> = const_rwlock(None);

/// Error returned by `try_register_spawner` indicating that a spawner was registered.
#[derive(Debug)]
pub struct SpawnerRegistered;

impl core::fmt::Display for SpawnerRegistered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "async_spawner: spawner already registered")
    }
}

impl std::error::Error for SpawnerRegistered {}

/// Try registering a spawner.
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
/// #[async_std::main]
/// async fn main() {
///     fn async_std_spawn(future: Boxed) -> Boxed {
///         Box::pin(async_std::task::spawn(future))
///     }
///     async_spawner::try_register_spawner(async_std_spawn).unwrap();
///     async_spawner::spawn(async {
///         println!("spawned on async-std");
///     });
/// }
/// ```
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
/// #[tokio::main]
/// async fn main() {
///     fn tokio_spawn(future: Boxed) -> Boxed {
///         Box::pin(async { tokio::spawn(future).await.unwrap() })
///     }
///     async_spawner::try_register_spawner(tokio_spawn).unwrap();
///     async_spawner::spawn(async {
///         println!("spawned on tokio");
///     });
/// }
/// ```
pub fn try_register_spawner(spawner: Spawner) -> Result<(), SpawnerRegistered> {
    let mut lock = SPAWNER.write();
    if lock.is_some() {
        return Err(SpawnerRegistered);
    }
    *lock = Some(spawner);
    Ok(())
}

/// Register a spawner.
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
/// #[async_std::main]
/// async fn main() {
///     fn async_std_spawn(future: Boxed) -> Boxed {
///         Box::pin(async_std::task::spawn(future))
///     }
///     async_spawner::register_spawner(async_std_spawn);
///     async_spawner::spawn(async {
///         println!("spawned on async-std");
///     });
/// }
/// ```
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
/// #[tokio::main]
/// async fn main() {
///     fn tokio_spawn(future: Boxed) -> Boxed {
///         Box::pin(async { tokio::spawn(future).await.unwrap() })
///     }
///     async_spawner::register_spawner(tokio_spawn);
///     async_spawner::spawn(async {
///         println!("spawned on tokio");
///     });
/// }
/// ```
pub fn register_spawner(spawner: Spawner) {
    try_register_spawner(spawner).unwrap();
}

/// Spawn a task.
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
/// #[async_std::main]
/// async fn main() {
///     fn async_std_spawn(future: Boxed) -> Boxed {
///         Box::pin(async_std::task::spawn(future))
///     }
///     async_spawner::register_spawner(async_std_spawn);
///     async_spawner::spawn(async {
///         println!("spawned on async-std");
///     });
/// }
/// ```
///
/// ```rust
/// # use core::future::Future;
/// # use core::pin::Pin;
/// # type Boxed = Pin<Box<dyn Future<Output = ()> + Send>>;
/// #[tokio::main]
/// async fn main() {
///     fn tokio_spawn(future: Boxed) -> Boxed {
///         Box::pin(async { tokio::spawn(future).await.unwrap() })
///     }
///     async_spawner::register_spawner(tokio_spawn);
///     async_spawner::spawn(async {
///         println!("spawned on tokio");
///     });
/// }
/// ```
pub fn spawn<F>(future: F) -> Boxed
where
    F: Future<Output = ()> + Send + 'static,
{
    SPAWNER.read().expect("async_spawn: no spawner registered")(Box::pin(future))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_async_std() {
        fn async_std_spawn(future: Boxed) -> Boxed {
            Box::pin(async_std::task::spawn(future))
        }
        try_register_spawner(async_std_spawn).ok();
        spawn(async {
            println!("spawned on async-std");
        });
    }

    #[tokio::test]
    async fn test_tokio() {
        fn tokio_spawn(future: Boxed) -> Boxed {
            Box::pin(async { tokio::spawn(future).await.unwrap() })
        }
        try_register_spawner(tokio_spawn).ok();
        spawn(async {
            println!("spawned on tokio");
        });
    }
}
