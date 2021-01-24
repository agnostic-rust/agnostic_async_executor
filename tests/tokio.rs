
#[cfg(test)]
mod tokio_tests {
    use agnostic_async_executor::tokio::tokio;
    use std::sync::Arc;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_spawn() {
        let exec = tokio();
        
        let res = exec.spawn(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test_spawn_blocking() {
        let exec = tokio();
        
        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test_spawn_local() {
        let exec = tokio();
        
        tokio::task::LocalSet::new().run_until(async {
                let res = exec.spawn_local(async {
                    1i32
                })
                .await;
                assert_eq!(res, 1);
        }).await;
    }
    #[tokio::test(flavor = "multi_thread")]
    #[ignore] // Until the issue in tokio is resolved
    async fn test_block_on() {
        let exec = Arc::new( tokio());
        
        let exec_clone = exec.clone();
        exec.spawn_blocking(move || {
            let res = exec_clone.block_on(async {
                1i32
            }).unwrap();
            assert_eq!(res, 1);
        }).await;
    }

    #[tokio::test]
    async fn test_spawn_st() {
        let exec = tokio();
        
        let res = exec.spawn(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    #[tokio::test]
    async fn test_spawn_blocking_st() {
        let exec = tokio();
        
        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    #[tokio::test]
    async fn test_spawn_local_st() {
        let exec = tokio();
        
        tokio::task::LocalSet::new().run_until(async {
                let res = exec.spawn_local(async {
                    1i32
                })
                .await;
                assert_eq!(res, 1);
        }).await;
    }
    #[tokio::test]
    #[ignore] // Until the issue in tokio is resolved
    async fn test_block_on_st() {
        let exec = Arc::new( tokio());
        
        let exec_clone = exec.clone();
        exec.spawn_blocking(move || {
            let res = exec_clone.block_on(async {
                1i32
            }).unwrap();
            assert_eq!(res, 1);
        }).await;
    }

}