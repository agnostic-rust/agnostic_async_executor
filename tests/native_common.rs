#![ cfg(not(feature = "wasm_bindgen_executor")) ]

mod common;

#[cfg(test)]
pub(crate) mod common_tests {
    use agnostic_async_executor::test::*;
    use super::common::common_tests;

    #[test]
    pub fn test_spawn() {
        test_in_native(false, common_tests::common_test_spawn);
    }

    #[test]
    pub fn test_spawn_blocking() {
        test_in_native(false, common_tests::common_test_spawn_blocking);
    }

    #[test]
    pub fn test_block_on() {
        test_in_native(false, common_tests::common_test_block_on);
    }

    #[test]
    pub fn test_sleep() {
        test_in_native(false, common_tests::common_test_sleep);
    }
    
    #[test]
    pub fn test_timeout() {
        test_in_native(false, common_tests::common_test_timeout);
    }

    #[test]
    pub fn test_interval() {
        test_in_native(false, common_tests::common_test_interval);
    }

    #[test]
    pub fn test_local() {
        test_in_native(false, common_tests::common_test_local);
    }

    #[test]
    pub fn test_cancel_handle() {
        test_in_native(false, common_tests::common_test_cancel_handle);
    }

    #[test]
    pub fn test_drop_handle() {
        test_in_native(false, common_tests::common_test_drop_handle);
    }

    #[test]
    pub fn test_global() {
        test_in_native(true, common_tests::common_test_global);
    }
    
}