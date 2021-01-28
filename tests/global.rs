#![ cfg(feature = "tokio_executor") ]

#[cfg(test)]
mod global_tests {

    use std::sync::Once;

    static INIT_GLOBAL_EXECUTOR: Once = Once::new();

    fn init() {
        INIT_GLOBAL_EXECUTOR.call_once(|| {
            let executor = agnostic_async_executor::new_agnostic_executor().use_tokio_executor();
            executor.set_as_global();
            std::thread::spawn(move || {
                executor.start(async {});
            });
        });
    }

    #[test]
    fn test_spawn_global() {
        init();
        agnostic_async_executor::spawn(async move {
            let res = agnostic_async_executor::spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_blocking_global() {
        init();
        agnostic_async_executor::spawn(async move {
            let res = agnostic_async_executor::spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_local_global() {
        init();
        agnostic_async_executor::spawn(async move {
            let res = agnostic_async_executor::spawn_local(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

}