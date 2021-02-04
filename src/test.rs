//! Test
//! TODO Doc

#![ cfg(feature = "test") ]

use std::sync::mpsc::{Sender, channel};

use crate::{AgnosticExecutor, AgnosticExecutorManager, new_agnostic_executor};

pub use super::check; // Because it's exported at the crate level, re re-export here for convenience

// TODO Create a crate for async testing, with the possibility of sending checks (optional message), debug messages, ...
// Include functions for handling durations and instants that work everywhere, including wasm
// Think about macros, including a macro for generating tests for multiple executors (including wasm?)

#[derive(Debug, Clone)]
enum TestMessage {
    Check(bool, String), 
}

use TestMessage::*;

/// TODO Doc
#[derive(Debug, Clone)]
pub struct TestHelper {
    runtime_name: String,
    executor: AgnosticExecutor,
    sender: Sender<TestMessage>
}

impl TestHelper {
    
    fn test_wrapper<F>(runtime_name: String, manager: AgnosticExecutorManager, errors: &mut Vec<String>, body: &F) where F: Fn(AgnosticExecutorManager, TestHelper) {
        let (sender, receiver) = channel();
        let executor = manager.get_executor();
        let helper = TestHelper {runtime_name, executor, sender};

        body(manager, helper);

        loop {
            match receiver.recv() {
                Ok(Check(success, msg)) => {
                    if !success {
                        errors.push(msg);
                    }
                }
                Err(_) => break,
            }
        }
    }

    /// TODO Doc
    pub fn get_executor(&self) -> &AgnosticExecutor {
        &self.executor
    }

    /// TODO Doc
    pub fn get_runtime_name(&self) -> &str {
        &self.runtime_name
    }

    /// TODO Doc
    pub fn check(&self, success: bool, msg: &str) {
        &self.sender.send(Check(success, msg.to_owned())).unwrap();
    }
}

/// TODO Doc
pub fn test_in_all<F>(body: F) where F: Fn(AgnosticExecutorManager, TestHelper) {
    let mut errors = Vec::new();

    #[ cfg(feature = "tokio_executor") ]
    {
        let manager = new_agnostic_executor().use_tokio_executor();
        TestHelper::test_wrapper("Tokio".to_owned(), manager, &mut errors, &body);
    }

    #[ cfg(feature = "async_std_executor") ]
    {
        let manager = new_agnostic_executor().use_async_std_executor();
        TestHelper::test_wrapper("AsyncStd".to_owned(), manager, &mut errors, &body);
    }

    #[ cfg(feature = "smol_executor") ]
    {
        let manager = new_agnostic_executor().use_smol_executor(None);
        TestHelper::test_wrapper("Smol".to_owned(), manager, &mut errors, &body);
    }

    #[ cfg(feature = "futures_executor") ]
    {
        let manager = new_agnostic_executor().use_futures_executor();
        TestHelper::test_wrapper("Futures".to_owned(), manager, &mut errors, &body);
    }

    #[ cfg(feature = "wasm_bindgen_executor") ]
    {
        let manager = new_agnostic_executor().use_wasm_bindgen_executor();
        TestHelper::test_wrapper("WasmBindgen".to_owned(), manager, &mut errors, &body);
    }

    if !errors.is_empty() {
        let msg = format!("\n{}\n", errors.join("\n"));
        assert!(false, msg);
    }
}

// TODO Test helpers for specific runtime tests test_in_X

/// TODO Doc
#[macro_export]
macro_rules! check {
    ($helper:expr, $val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                let msg = format!("Failed check [{}:{}] {} = {:#?} // Using {} runtime", file!(), line!(), stringify!($val), &tmp, $helper.get_runtime_name());
                $helper.check(tmp, &msg);
                tmp
            }
        }
    };
}
