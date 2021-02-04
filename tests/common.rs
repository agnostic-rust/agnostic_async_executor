#[cfg(test)]
pub(crate) mod common_tests {
    use agnostic_async_executor::{Stopwatch, test::*};

    #[test]
    pub fn test_spawn() {
        test_in_all(|manager, mut helper| {
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
    pub fn test_spawn_blocking() {
        test_in_all(|manager, mut helper| {
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
    pub fn test_sleep() {
        test_in_all(|manager, mut helper| {
            manager.start(async move{
                let exec = helper.get_executor();
                let sw = Stopwatch::new_tolerant_millis(2);
                exec.sleep(std::time::Duration::from_millis(200)).await;
                check!(helper, sw.has_elapsed_millis(200));
            });
        });
    }
    
    #[test]
    pub fn test_timeout() {
        test_in_all(|manager, mut helper| {
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
    pub fn test_interval() {
        test_in_all(|manager, mut helper| {
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
        });
    }
}