#![cfg(not(feature = "wasm_bindgen_executor"))]

use agnostic_async_executor::AgnosticExecutorManager;

// TODO Create a crate for async testing, with the possibility of sending checks (optional message), debug messages, ...
// Include functions for handling durations and instants that work everywhere, including wasm
// Think about macros, including a macro for generating tests for multiple executors (including wasm?)

fn check_errors(rx: std::sync::mpsc::Receiver<(bool, &str)>) {
    loop {
        match rx.recv() {
            Ok((success, msg)) => {
                assert!(success, msg.to_owned());
            }
            Err(_) => break,
        }
    }
}

pub fn test_spawn(manager: AgnosticExecutorManager) {
    let (tx, rx) = std::sync::mpsc::channel();
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.spawn(async {
            1i32
        }).await;
        tx.send((res == 1, "Error spawning")).unwrap();
    });

    check_errors(rx);  
}

pub fn test_spawn_blocking(manager: AgnosticExecutorManager) {
    let (tx, rx) = std::sync::mpsc::channel();
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.spawn_blocking(|| {
            1i32
        }).await;
        tx.send((res == 1, "Error spawning blocking")).unwrap();
    });

    check_errors(rx);  
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
    let (tx, rx) = std::sync::mpsc::channel();
    let exec = manager.get_executor();
    manager.start(async move {
        let start = std::time::Instant::now();
        exec.sleep(std::time::Duration::from_millis(200)).await;
        let dur = std::time::Instant::now().checked_duration_since(start).unwrap().as_millis();
        let margin = 2;
        tx.send((dur + margin >= 200, "Error sleeping")).unwrap();
    });
    check_errors(rx);
}

pub fn test_timeout(manager: AgnosticExecutorManager) {
    let (tx, rx) = std::sync::mpsc::channel();
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.timeout(std::time::Duration::from_millis(100), async {
            exec.sleep(std::time::Duration::from_millis(200)).await
        }).await;
        tx.send((res.is_err(), "Error on timeout")).unwrap();
    });
    check_errors(rx);
}

pub fn test_interval(manager: AgnosticExecutorManager) {
    let (tx, rx) = std::sync::mpsc::channel();
    let exec = manager.get_executor();
    manager.start(async move {
        let start = std::time::Instant::now();

        let delay = 10;
        let margin = 2.0;

        let mut interval = exec.interval(std::time::Duration::from_millis(delay));

        for i in 1..100u64 {
            interval.next().await;
    
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    
            tx.send((elapsed + margin >= (i*delay) as f64, "Error on interval")).unwrap();
        }
    });
    check_errors(rx);
}