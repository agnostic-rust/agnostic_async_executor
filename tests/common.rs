#![cfg(not(feature = "wasm_bindgen_executor"))]

use agnostic_async_executor::AgnosticExecutorManager;

pub fn test_spawn(manager: AgnosticExecutorManager) {
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.spawn(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    });
}

pub fn test_spawn_blocking(manager: AgnosticExecutorManager) {
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        assert_eq!(res, 1);
    });
}

// pub fn test_spawn_local(manager: AgnosticExecutorManager) {
//     let exec = manager.get_executor();
//     manager.start(async move {
//         let res = exec.spawn_local(async {
//             1i32
//         }).await;
//         assert_eq!(res, 1);
//     });
// }

pub fn test_sleep(manager: AgnosticExecutorManager) {
    let exec = manager.get_executor();
    manager.start(async move {
        let start = std::time::Instant::now();
        exec.sleep(std::time::Duration::from_millis(200)).await;
        let dur = std::time::Instant::now().checked_duration_since(start).unwrap().as_millis();
        assert!(dur >= 200);
    });
}

pub fn test_timeout(manager: AgnosticExecutorManager) {
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.timeout(std::time::Duration::from_millis(100), async {
            exec.sleep(std::time::Duration::from_millis(200)).await
        }).await;
        assert!(res.is_err());
    });
}