pub(crate) mod common_tests {
    use agnostic_async_executor::{AgnosticExecutorManager, test::*, time::Stopwatch};

    pub fn common_test_spawn(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        manager.start(async move{
            let exec = helper.get_executor();
            let res = exec.spawn(async {
                1i32
            }).await;
            check!(helper, res == 1);
        });
    }

    pub fn common_test_spawn_blocking(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        manager.start(async move{
            let exec = helper.get_executor();
            let res = exec.spawn_blocking(|| {
                1i32
            }).await;
            check!(helper, res == 1);
        });
    }

    pub fn common_test_sleep(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        manager.start(async move{
            let exec = helper.get_executor();
            let sw = Stopwatch::new_tolerant_millis(2);
            exec.sleep(std::time::Duration::from_millis(200)).await;
            check!(helper, sw.has_elapsed_millis(200));
        });
    }

    pub fn common_test_timeout(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        manager.start(async move{
            let exec = helper.get_executor();
            let res = exec.timeout(std::time::Duration::from_millis(100), async {
                exec.sleep(std::time::Duration::from_millis(200)).await
            }).await;
            check!(helper, res.is_err());
        });
    }

    pub fn common_test_interval(manager: AgnosticExecutorManager, mut helper: TestHelper) {
        manager.start(async move{
            let exec = helper.get_executor();
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
}