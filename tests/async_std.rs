#![ cfg(feature = "async_std_executor") ]

#[cfg(test)]
mod async_std_tests {
    use agnostic_async_executor::async_std_executor;

    #[test]
    fn test_spawn() {
        let exec = async_std_executor();
        exec.start(async move {
            let res = exec.spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_blocking() {
        let exec = async_std_executor();
        exec.start(async move {
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_local() {
        let exec = async_std_executor();
        exec.start(async move {
            let res = exec.spawn_local(async {
                1i32
            })
            .await;
            assert_eq!(res, 1);
        });
    }
}