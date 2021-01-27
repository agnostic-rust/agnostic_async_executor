
#![ cfg(all( feature = "wasm_bindgen_executor", target_arch = "wasm32" )) ]

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

#[cfg(test)]
mod async_std_tests {
    use agnostic_async_executor::wasm_bindgen;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_spawn() {
        let exec = wasm_bindgen();
        
        let res = exec.spawn(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    }
    
    #[wasm_bindgen_test]
    async fn test_spawn_blocking() {
        let exec = wasm_bindgen();
        
        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        assert_eq!(res, 1);
    }

    #[wasm_bindgen_test]
    async fn test_spawn_local() {
        let exec = wasm_bindgen();
        
        let res = exec.spawn_local(async {
            1i32
        })
        .await;
        assert_eq!(res, 1);
    }

}