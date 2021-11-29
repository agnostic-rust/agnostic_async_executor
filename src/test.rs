//! Agnostic executor test utilities that can be used to test your executor agnostic async code in all the available executors with minimal effort, including in wasm. 

#![ cfg(feature = "test") ]

use std::sync::Arc;

use concurrent_queue::ConcurrentQueue;

use crate::{AgnosticExecutorManager, new_agnostic_executor, check_global_executor};

pub use super::{check, check_eq, check_op, check_gt, check_lt, check_ge, check_le}; // Because it's exported at the crate level, re-export  it here for convenience

#[derive(Debug, Clone)]
enum TestMessage {
    Check(bool, String), 
}

use TestMessage::*;

/// A test helper instance that can be used to perform checks inside async tasks of the tests in a ways that is executor agnostic and works everywhere, even in wasm
#[derive(Debug, Clone)]
pub struct TestHelper {
    runtime_name: String,
    test_queue: Arc<ConcurrentQueue<TestMessage>>
}

impl TestHelper {
    
    fn test_wrapper_native<F>(runtime_name: String, manager: AgnosticExecutorManager, errors: &mut Vec<String>, body: &F) where F: Fn(AgnosticExecutorManager, TestHelper) {
        let test_queue = Arc::new(ConcurrentQueue::unbounded());
        let helper = TestHelper {runtime_name, test_queue: test_queue.clone()};

        body(manager, helper);

        // IMPORTANT This assumes that manager.start is a blocking call on native platforms (unlike wasm)

        loop {
            match test_queue.pop() {
                Ok(Check(success, msg)) => {
                    if !success {
                        errors.push(msg);
                    }
                },
                Err(_) => break
            }
        }
    }

    /// Get the runtime name being used in this test
    pub fn get_runtime_name(&self) -> &str {
        &self.runtime_name
    }

    /// Perform a check. Usually is prefered to use the provided check*! macros to get better error messages
    /// You should always use checks instead of asserts if you want to be sure errors are cached in all executors and situations, specially but not only in wasm. 
    pub fn check(&mut self, success: bool, msg: &str) {
        let _ = &self.test_queue.push(Check(success, msg.to_owned()));
    }
}

/// Define and run a native test that will be executed on all the configured executors except wasm, that needs it's own test
#[ cfg(not(feature = "wasm_bindgen_executor")) ]
pub fn test_in_native<F>(global: bool, body: F) where F: Fn(AgnosticExecutorManager, TestHelper) {
    let mut errors = Vec::new();

    // As we can only have one global executor, we only test tokio that is the one that has more restrictions and it's the first

    if global {
        #[ cfg(feature = "tokio_executor") ]
        {
            let manager = new_agnostic_executor().use_tokio_executor();
            if !check_global_executor() { manager.set_as_global() }
            TestHelper::test_wrapper_native("Tokio".to_owned(), manager, &mut errors, &body);
        }
    } else {
        #[ cfg(feature = "tokio_executor") ]
        {
            let manager = new_agnostic_executor().use_tokio_executor();
            TestHelper::test_wrapper_native("Tokio".to_owned(), manager, &mut errors, &body);
        }
        #[ cfg(feature = "async_std_executor") ]
        {
            let manager = new_agnostic_executor().use_async_std_executor();
            TestHelper::test_wrapper_native("AsyncStd".to_owned(), manager, &mut errors, &body);
        }

        #[ cfg(feature = "smol_executor") ]
        {
            let manager = new_agnostic_executor().use_smol_executor(None);
            TestHelper::test_wrapper_native("Smol".to_owned(), manager, &mut errors, &body);
        }

        #[ cfg(feature = "futures_executor") ]
        {
            let manager = new_agnostic_executor().use_futures_executor();
            TestHelper::test_wrapper_native("Futures".to_owned(), manager, &mut errors, &body);
        }
    }

    let without_errors = errors.is_empty();
    if !without_errors {
        let msg = format!("\n{}\n", errors.join("\n"));
        assert!(without_errors, "{}", msg);
    }
}

/// Define and un a wasm test
#[ cfg(feature = "wasm_bindgen_executor") ]
pub async fn test_in_wasm<F>(body: F) where F: Fn(AgnosticExecutorManager, TestHelper) {

    let mut manager = new_agnostic_executor().use_wasm_bindgen_executor();

    if !check_global_executor() { manager.set_as_global() }

    let test_queue = Arc::new(ConcurrentQueue::unbounded());

    let (sender, receiver) = futures::channel::oneshot::channel::<i32>();

    manager.on_finish(|| { sender.send(1).unwrap();  } );

    let helper = TestHelper {runtime_name: "WasmBindgen".to_owned(), test_queue: test_queue.clone()};

    body(manager, helper);

    receiver.await.unwrap();

    let mut errors = Vec::new();
    
    loop {
        match test_queue.pop() {
            Ok(Check(success, msg)) => {
                if !success {
                    errors.push(msg);
                }
            },
            Err(_) => break
        }
    }

    let without_errors = errors.is_empty();
    if !without_errors {
        let msg = format!("\n{}\n", errors.join("\n"));
        assert!(without_errors, "{}", msg);
    }
}

/// A macro to perform a check inside an agnostic executor test, this is the preferred way to check because it provides better information if the check fails.
/// You should always use checks instead of asserts if you want to be sure errors are cached in all executors and situations, specially but not only in wasm.
/// TODO Example
#[macro_export]
macro_rules! check {
    ($helper:expr, $val:expr) => {
        match $val {
            tmp => {
                let msg = format!("Failed check [{}:{}] {} // Using {} runtime", file!(), line!(), stringify!($val), $helper.get_runtime_name());
                $helper.check(tmp, &msg);
                tmp
            }
        }
    };
}

/// A check macro variant that accepts any binary check operation while providing useful information
/// TODO Example
#[macro_export]
macro_rules! check_op {
    ($helper:expr, $a:expr, $b:expr, $op:tt) => {
        match ($a, $b) {
            (tmp_a, tmp_b) => {
                let res = $op($a, $b);
                let msg = format!("Failed check_op [{}:{}] a:{} = {:#?}; b:{} = {:#?}; op:{} // Using {} runtime", file!(), line!(), stringify!($a), &tmp_a, stringify!($b), &tmp_b, stringify!($op), $helper.get_runtime_name());
                $helper.check(res, &msg);
            }
        }
    };
}

/// A check macro variant that performs the a == b operation while providing useful information
/// TODO Example
#[macro_export]
macro_rules! check_eq {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, (|a, b| a == b ));   
    }
}

/// A check macro variant that performs the a > b operation while providing useful information
/// TODO Example
#[macro_export]
macro_rules! check_gt {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, (|a, b| a > b ));   
    }
}

/// A check macro variant that performs the a < b operation while providing useful information
/// TODO Example
#[macro_export]
macro_rules! check_lt {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, (|a, b| a < b ));   
    }
}

/// A check macro variant that performs the a >= b operation while providing useful information
/// TODO Example
#[macro_export]
macro_rules! check_ge {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, (|a, b| a >= b ));   
    }
}

/// A check macro variant that performs the a <= b operation while providing useful information
/// TODO Example
#[macro_export]
macro_rules! check_le {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, (|a, b| a <= b ));   
    }
}