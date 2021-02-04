#![cfg(not(feature = "wasm_bindgen_executor"))]

#[cfg(test)]
mod common_tests {
    use agnostic_async_executor::test::*;

    #[test]
    fn test_spawn() {
        test_in_all(|manager, helper| {
            manager.start(async move{
                let exec = helper.get_executor();
                let res = exec.spawn(async {
                    1i32
                }).await;
                check!(helper, res == 1);
            });
        });
    }

    #[test]
    fn test_spawn_blocking() {
        test_in_all(|manager, helper| {
            manager.start(async move{
                let exec = helper.get_executor();
                let res = exec.spawn_blocking(|| {
                    1i32
                }).await;
                check!(helper, res == 1);
            });
        });
    }

    #[test]
    fn test_sleep() {
        test_in_all(|manager, helper| {
            manager.start(async move{
                let exec = helper.get_executor();
                let start = std::time::Instant::now();
                exec.sleep(std::time::Duration::from_millis(200)).await;
                let dur = std::time::Instant::now().checked_duration_since(start).unwrap().as_millis();
                let margin = 2;
                check!(helper, dur + margin >= 200);
            });
        });
    }
    
    #[test]
    fn test_timeout() {
        test_in_all(|manager, helper| {
            manager.start(async move{
                let exec = helper.get_executor();
                let res = exec.timeout(std::time::Duration::from_millis(100), async {
                    exec.sleep(std::time::Duration::from_millis(200)).await
                }).await;
                check!(helper, res.is_err());
            });
        });
    }

    #[test]
    fn test_interval() {
        test_in_all(|manager, helper| {
            manager.start(async move{
                let exec = helper.get_executor();
                let start = std::time::Instant::now();
                let delay = 10;
                let margin = 2.0;
                let mut interval = exec.interval(std::time::Duration::from_millis(delay));

                for i in 1..100u64 {
                    interval.next().await;
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    check!(helper, elapsed + margin >= (i*delay) as f64);
                }
            });
        });
    }

    // // We should do this for wasm, but we need stopwatch first !!!
    // #[test] 
    // fn test_spawn_2() {
    //     test_spawn();
    // }

}