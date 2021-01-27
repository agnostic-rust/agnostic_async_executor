#![ cfg(feature = "smol_executor") ]

#[cfg(test)]
mod smol_tests {
    use agnostic_async_executor::smol;
    use std::sync::Arc;

    #[test]
    fn test_spawn() {
        let exec = smol();
        exec.block_on(async move {
            let res = exec.spawn(async {
                1i32
            }).await;
            assert_eq!(res, 1);
        }).unwrap();
    }

    #[test]
    fn test_spawn_blocking() {
        let exec = smol();
        exec.block_on(async move {
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            assert_eq!(res, 1);
        }).unwrap();
    }

    #[test]
    fn test_spawn_local() {
        let exec = smol();
        exec.block_on(async move {
            let res = exec.spawn_local(async {
                1i32
            })
            .await;
            assert_eq!(res, 1);
        }).unwrap();
    }

    #[test]
    fn test_block_on() {
        let exec = Arc::new( smol());
        
        let exec_clone = exec.clone();
        exec.block_on(async move {
            let res = exec_clone.block_on(async {
                1i32
            }).unwrap();
            assert_eq!(res, 1);
        }).unwrap();
    }

}