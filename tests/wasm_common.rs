#![ cfg(all( feature = "wasm_bindgen_executor", target_arch = "wasm32" )) ]

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

mod common;

#[cfg(test)]
mod wasm_common_tests {
    use wasm_bindgen_test::*;
    use agnostic_async_executor::test::*;
    use super::common::common_tests;

    #[wasm_bindgen_test]
    async fn test_spawn() {
        test_in_wasm(common_tests::common_test_spawn).await;
    }

    #[wasm_bindgen_test]
    async fn test_spawn_blocking() {
        test_in_wasm(common_tests::common_test_spawn_blocking).await;
    }

    
    #[wasm_bindgen_test]
    async fn test_spawn_global() {
        test_in_wasm(common_tests::common_test_spawn_global).await;
    }

    #[wasm_bindgen_test]
    async fn test_spawn_blocking_global() {
        test_in_wasm(common_tests::common_test_spawn_blocking_global).await;
    }

    #[wasm_bindgen_test]
    async fn test_sleep() {
        test_in_wasm(common_tests::common_test_sleep).await;
    }

    #[wasm_bindgen_test]
    async fn test_timeout() {
        test_in_wasm(common_tests::common_test_timeout).await;
    }

    #[wasm_bindgen_test]
    async fn test_interval() {
        test_in_wasm(common_tests::common_test_interval).await;
    }

    #[wasm_bindgen_test]
    async fn test_local() {
        test_in_wasm(common_tests::common_test_local).await;
    }

}