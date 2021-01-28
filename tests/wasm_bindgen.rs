
#![ cfg(all( feature = "wasm_bindgen_executor", target_arch = "wasm32" )) ]

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

mod common;

#[cfg(test)]
mod wasm_bindgen_tests {
    use agnostic_async_executor::{AgnosticExecutorManager, new_agnostic_executor};
    use wasm_bindgen_test::*;

    fn get_manager() -> AgnosticExecutorManager {
        new_agnostic_executor().use_wasm_bindgen_executor()
    }

    #[wasm_bindgen_test]
    fn test_spawn() {
        super::common::test_spawn(get_manager());
    }

    #[wasm_bindgen_test]
    fn test_spawn_blocking() {
        super::common::test_spawn_blocking(get_manager());
    }

    #[wasm_bindgen_test]
    fn test_spawn_local() {
        super::common::test_spawn_local(get_manager());
    }
}