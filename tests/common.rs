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

pub fn test_spawn_local(manager: AgnosticExecutorManager) {
    let exec = manager.get_executor();
    manager.start(async move {
        let res = exec.spawn_local(async {
            1i32
        }).await;
        assert_eq!(res, 1);
    });
}