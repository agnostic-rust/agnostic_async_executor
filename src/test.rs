//! Test
//! TODO Doc

#![ cfg(feature = "test") ]

use futures::{SinkExt, channel::mpsc::{UnboundedSender, unbounded}};

use crate::{AgnosticExecutor, AgnosticExecutorManager, new_agnostic_executor};

pub use super::{check, check_op, check_gt, check_lt, check_ge, check_le}; // Because it's exported at the crate level, re re-export here for convenience

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
    sender: UnboundedSender<TestMessage>
}

impl TestHelper {
    
    fn test_wrapper<F>(runtime_name: String, manager: AgnosticExecutorManager, errors: &mut Vec<String>, body: &F) where F: Fn(AgnosticExecutorManager, TestHelper) {
        let (sender, mut receiver) = unbounded();
        let executor = manager.get_executor();
        let helper = TestHelper {runtime_name, executor, sender};

        body(manager, helper);

        loop {
            match receiver.try_next() {
                Ok(Some(Check(success, msg))) => {
                    if !success {
                        errors.push(msg);
                    }
                },
                _ => break
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
    pub fn check(&mut self, success: bool, msg: &str) {
        &self.sender.send(Check(success, msg.to_owned()));
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
                let msg = format!("Failed check [{}:{}] {} // Using {} runtime", file!(), line!(), stringify!($val), $helper.get_runtime_name());
                $helper.check(tmp, &msg);
                tmp
            }
        }
    };
}

/// TODO Doc
#[macro_export]
macro_rules! check_op {
    ($helper:expr, $a:expr, $b:expr, $op:tt) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match ($a, $b) {
            (tmp_a, tmp_b) => {
                let res = $op($a, $b);
                let msg = format!("Failed check_op [{}:{}] a:{} = {:#?}; b:{} = {:#?}; op:{} // Using {} runtime", file!(), line!(), stringify!($a), &tmp_a, stringify!($b), &tmp_b, stringify!($op), $helper.get_runtime_name());
                $helper.check(res, &msg);
            }
        }
    };
}

/// TODO Doc
pub fn check_gt<A: PartialOrd<B>, B>(a: A, b: B) -> bool { a > b }

/// TODO Doc
#[macro_export]
macro_rules! check_gt {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, check_gt);   
    }
}

/// TODO Doc
pub fn check_lt<A: PartialOrd<B>, B>(a: A, b: B) -> bool { a < b }

/// TODO Doc
#[macro_export]
macro_rules! check_lt {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, check_lt);   
    }
}

/// TODO Doc
pub fn check_ge<A: PartialOrd<B>, B>(a: A, b: B) -> bool { a >= b }

/// TODO Doc
#[macro_export]
macro_rules! check_ge {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, check_ge);   
    }
}

/// TODO Doc
pub fn check_le<A: PartialOrd<B>, B>(a: A, b: B) -> bool { a <= b }

/// TODO Doc
#[macro_export]
macro_rules! check_le {
    ($helper:expr, $a:expr, $b:expr) => {
        agnostic_async_executor::test::check_op!($helper, $a, $b, check_le);   
    }
}