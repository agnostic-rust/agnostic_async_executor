
#![ cfg(all( feature = "wasm_bindgen_executor", target_arch = "wasm32" )) ]

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

mod common;

#[cfg(test)]
mod wasm_common_tests {
    use wasm_bindgen_test::*;
    use super::common::common_tests;

    #[wasm_bindgen_test]
    fn test_spawn() {
        common_tests::test_spawn();
    }

    #[wasm_bindgen_test]
    fn test_spawn_blocking() {
        common_tests::test_spawn_blocking();
    }

    #[wasm_bindgen_test]
    fn test_sleep() {
        common_tests::test_sleep();
    }

    #[wasm_bindgen_test]
    fn test_timeout() {
        common_tests::test_timeout();
    }

    #[wasm_bindgen_test]
    fn test_interval() {
        common_tests::test_interval();
    }

}