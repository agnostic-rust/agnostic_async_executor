#![ cfg(feature = "futures_executor") ]

#[cfg(test)]
mod futures_tests {
    use agnostic_async_executor::futures_executor;

    #[test]
    fn test_spawn() {
        let exec = futures_executor();
        exec.start(async move {
            let res = exec.spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_blocking() {
        let exec = futures_executor();
        exec.start(async move {
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_local() {
        let exec = futures_executor();
        exec.start(async move {
            let res = exec.spawn_local(async {
                1i32
            })
            .await;
            assert_eq!(res, 1);
        });
    }
}