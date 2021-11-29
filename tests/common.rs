pub(crate) mod common_tests {
    use agnostic_async_executor::{AgnosticExecutorManager, test::*, time::Stopwatch};
    use futures::channel::oneshot;

    pub fn common_test_spawn(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{
            let res = exec.spawn(async {
                1i32
            }).await;
            check!(helper, res == 1);
        });
    }

    pub fn common_test_spawn_blocking(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            check!(helper, res == 1);
        });
    }

    #[cfg(feature = "block_on")]
    pub fn common_test_block_on(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();        
        let res = exec.block_on(async {
            1i32
        });
        check!(helper, res == 1);
    }

    pub fn common_test_sleep(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{
            let sw = Stopwatch::new_tolerant_millis(2);
            exec.sleep(std::time::Duration::from_millis(200)).await;
            check!(helper, sw.has_elapsed_millis(200));
        });
    }

    pub fn common_test_timeout(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{
            let res = exec.timeout(std::time::Duration::from_millis(100), async {
                exec.sleep(std::time::Duration::from_millis(200)).await
            }).await;
            check!(helper, res.is_err());
        });
    }

    pub fn common_test_interval(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{
            let sw = Stopwatch::new_tolerant_millis(2);
            let delay = 10;
            let mut interval = exec.interval(std::time::Duration::from_millis(delay));

            for i in 1..100u64 {
                interval.next().await;
                check!(helper, sw.has_elapsed_millis(i*delay));
            }
        });
    }

    pub fn common_test_local(mut manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let not_send_number = std::rc::Rc::new(1i32);
        let send_number = std::sync::Arc::new(2i32);

        let exec =  manager.get_executor();
        let local = manager.get_local_executor();
        manager.start(async move {
            let res = local.spawn_local(async move {
                *not_send_number + 1
            }).await;

            check!(helper, res == 2);

            let res = exec.spawn(async move {
                *send_number + 1
            }).await;

            check!(helper, res == 3);
        });
    }

    pub fn common_test_cancel_handle(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{

            let (c_tx, c_rx) = oneshot::channel();

            let exec2 = exec.clone();

            let res = exec.spawn(async move {
                exec2.sleep(std::time::Duration::from_millis(200)).await; // Give some time for the cancellation to occur
                c_tx.send(1i32).unwrap();
            });

            res.cancel().await;

            let channel_res = c_rx.await;

            check!(helper, channel_res.is_err());
        });
    }

    pub fn common_test_drop_handle(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        let exec = manager.get_executor();
        manager.start(async move{

            let (c_tx, c_rx) = oneshot::channel();

            let exec2 = exec.clone();

            let res = exec.spawn(async move {
                exec2.sleep(std::time::Duration::from_millis(200)).await; // Give some time for the cancellation to occur if there is any bug
                c_tx.send(1i32).unwrap();
            });

            drop(res);

            let channel_res = c_rx.await;

            check!(helper, channel_res.is_ok());

            if let Ok(channel_res) = channel_res {
                check!(helper, channel_res == 1);
            }
        });
    }

    pub fn common_test_global(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        // We test global on a single test, because otherwise we would get mismatching global executors, as they can only be set once

        // Block On
        #[cfg(feature = "block_on")]
        {
            let res = agnostic_async_executor::block_on(async {
                1i32
            });
            check!(helper, res == 1);
        } 

        manager.start(async move{
            // Spawn
            let res = agnostic_async_executor::spawn(async {
                1i32
            }).await;
            check!(helper, res == 1);

            // Spawn Blocking
            let res = agnostic_async_executor::spawn_blocking(|| {
                1i32
            }).await;
            check!(helper, res == 1);
        });
    }
}