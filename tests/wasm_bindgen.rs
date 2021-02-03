
#![ cfg(all( feature = "wasm_bindgen_executor", target_arch = "wasm32" )) ]

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

#[cfg(test)]
mod wasm_bindgen_tests {
    use agnostic_async_executor::{AgnosticExecutor, new_agnostic_executor};
    use wasm_bindgen_test::*;
    use agnostic_async_executor::wasm_now;

    // We get the executor directly, without starting the manager, because it breaks wasm_bindgen_test and cannot capture the asserts
    fn get_executor() -> AgnosticExecutor {
        new_agnostic_executor().use_wasm_bindgen_executor().get_executor()
    }

    #[wasm_bindgen_test]
    async fn test_spawn() {
        let exec = get_executor();

        let res = exec.spawn(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    }

    #[wasm_bindgen_test]
    async fn test_spawn_blocking() {
        let exec = get_executor();

        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        assert_eq!(res, 1);
    }

    // #[wasm_bindgen_test]
    // async fn test_spawn() {
    //     let exec = get_executor();

    //     let res = exec.spawn_local(async {
    //         1i32
    //     }).await;
    //     assert_eq!(res, 1);
    // }

    #[wasm_bindgen_test]
    async fn test_sleep() {
        let exec = get_executor();
        let start = wasm_now();
        exec.sleep(std::time::Duration::from_millis(200)).await;
        let dur = wasm_now() - start;
        assert!(dur >= 200.0);
    }

    #[wasm_bindgen_test]
    async fn test_timeout() {
        let exec = get_executor();

        let res = exec.timeout(std::time::Duration::from_millis(100), async {
            exec.sleep(std::time::Duration::from_millis(200)).await;
        }).await;

        assert!(res.is_err());
        
        let res = exec.timeout(std::time::Duration::from_millis(100), async {
            exec.sleep(std::time::Duration::from_millis(50)).await;
            1u32
        }).await;

        assert_eq!(res.unwrap(), 1);
    }

    #[wasm_bindgen_test]
    async fn test_interval() {
        let exec = get_executor();

        let start = wasm_now();

        let delay = 10;
        let margin = 2.0;

        let mut interval = exec.interval(std::time::Duration::from_millis(delay));

        for i in 1..100u64 {
            interval.next().await;
    
            let current = wasm_now();
    
            assert!(current + margin >= start + (i*delay) as f64);
        }
    }
}