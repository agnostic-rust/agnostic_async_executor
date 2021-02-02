#![ cfg(feature = "tokio_executor") ]

#[cfg(test)]
mod global_tests {

    #[test]
    fn test_global() {
        let executor = agnostic_async_executor::new_agnostic_executor().use_tokio_executor();
        executor.set_as_global();

        executor.start(async {

            let res = agnostic_async_executor::spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);

            let res = agnostic_async_executor::spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);

            let res = agnostic_async_executor::spawn_local(async {
                1i32
            }).await;
            assert_eq!(res, 1);

        });
    }

}