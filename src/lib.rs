//! Agnostic Async Executor
//! TODO Doc

#![deny(missing_docs)]

mod executors;

pub use executors::{
    JoinHandle, AgnosticExecutor, AgnosticExecutorBuilder, AgnosticExecutorManager,
    new_agnostic_executor, get_global_executor, spawn, spawn_blocking//, spawn_local
};

#[ cfg(feature = "time") ]
pub mod time;

#[ cfg(feature = "test") ]
pub mod test;