#![ cfg(feature = "smol_executor") ]

mod common;

#[cfg(test)]
mod smol_tests {
    use agnostic_async_executor::{AgnosticExecutorManager, new_agnostic_executor};

    fn get_manager() -> AgnosticExecutorManager {
        new_agnostic_executor().use_smol_executor()
    }

    #[test]
    fn test_spawn() {
        super::common::test_spawn(get_manager());
    }

    #[test]
    fn test_spawn_blocking() {
        super::common::test_spawn_blocking(get_manager());
    }

    #[test]
    fn test_spawn_local() {
        super::common::test_spawn_local(get_manager());
    }
}