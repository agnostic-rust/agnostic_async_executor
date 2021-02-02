
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

    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(inline_js = r#"
    export function now() {
        return performance.now();
    }"#)]
    extern "C" {
        fn now() -> f64;
    }

    #[wasm_bindgen_test]
    fn test_sleep() {
        let manager = get_manager();
        let exec = manager.get_executor();
        manager.start(async move {
            let start = now();
            console_log!("Start {}", start);
            exec.sleep(std::time::Duration::from_millis(200)).await;
            let dur = now() - start;
            console_log!("Dur {}", dur);
            // assert!(dur >= 200.0);
        });
    }

    // #[wasm_bindgen_test]
    // fn test_timeout() {
    //     let manager = get_manager();
    //     let exec = manager.get_executor();
    //     manager.start(async move {
    //         let res = exec.timeout(std::time::Duration::from_millis(100), async {
    //             console_log!("Test 1");
    //             let start = std::time::Instant::now();
    //             exec.sleep(std::time::Duration::from_millis(200)).await;
    //             let dur = std::time::Instant::now().checked_duration_since(start).unwrap().as_millis();
    //             console_log!("Test 2: {}", dur);
    //         }).await;
    //         assert!(res.is_err());
    //     });
    // }
}