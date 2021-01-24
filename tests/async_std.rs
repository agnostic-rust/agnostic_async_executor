
#[cfg(test)]
mod async_std_tests {
    use agnostic_async_executor::async_std::async_std;
    use std::sync::Arc;

    #[async_std::test]
    async fn test_spawn() {
        let exec = async_std();
        
        let res = exec.spawn(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    #[async_std::test]
    async fn test_spawn_blocking() {
        let exec = async_std();
        
        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    #[async_std::test]
    async fn test_spawn_local() {
        let exec = async_std();
        
            let res = exec.spawn_local(async {
                1i32
            })
            .await;
            assert_eq!(res, 1);
    }
    #[async_std::test]
    async fn test_block_on() {
        let exec = Arc::new( async_std());
        
        let exec_clone = exec.clone();
        exec.spawn_blocking(move || {
            let res = exec_clone.block_on(async {
                1i32
            }).unwrap();
            assert_eq!(res, 1);
        }).await;
    }

}