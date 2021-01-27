#![ cfg(feature = "smol_executor") ]

#[cfg(test)]
mod smol_tests {
    use agnostic_async_executor::smol_executor;

    #[test]
    fn test_spawn() {
        let exec = smol_executor();
        exec.clone().start(async move {
            let res = exec.spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_blocking() {
        let exec = smol_executor();
        exec.clone().start(async move {
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);
        });
    }

    #[test]
    fn test_spawn_local() {
        let exec = smol_executor();
        exec.clone().start(async move {
            let res = exec.spawn_local(async {
                1i32
            })
            .await;
            assert_eq!(res, 1);
        });
    }

}