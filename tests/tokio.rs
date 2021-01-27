#![ cfg(feature = "tokio_executor") ]

#[cfg(test)]
mod tokio_tests {
    use agnostic_async_executor::tokio_executor;

    #[test]
    fn test_spawn() {
        let exec = tokio_executor();
        exec.clone().start(async move {
            let res = exec.spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_blocking() {
        let exec = tokio_executor();
        exec.clone().start(async move {
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_local() {
        let exec = tokio_executor();
        exec.clone().start(async move {
            let res = exec.spawn_local(async {
                1i32
            })
            .await;
            assert_eq!(res, 1);
        });
    }
}