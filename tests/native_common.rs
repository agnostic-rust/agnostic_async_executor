#![ cfg(not(feature = "wasm_bindgen_executor")) ]

mod common;

#[cfg(test)]
pub(crate) mod common_tests {
    use agnostic_async_executor::test::*;
    use super::common::common_tests;

    #[test]
    pub fn test_spawn() {
        test_in_native(common_tests::common_test_spawn);
    }

    #[test]
    pub fn test_spawn_blocking() {
        test_in_native(common_tests::common_test_spawn_blocking);
    }

    #[test]
    pub fn test_sleep() {
        test_in_native(common_tests::common_test_sleep);
    }
    
    #[test]
    pub fn test_timeout() {
        test_in_native(common_tests::common_test_timeout);
    }

    #[test]
    pub fn test_interval() {
        test_in_native(common_tests::common_test_interval);
    }

    #[test]
    pub fn test_local() {
        test_in_native(common_tests::common_test_local);
    }
}