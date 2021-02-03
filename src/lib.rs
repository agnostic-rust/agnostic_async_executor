//! Agnostic Async Executor
//! TODO Doc

#![deny(missing_docs)]
#![deny(warnings)]

mod agnostic_executor;

pub use agnostic_executor::{
    JoinHandle, AgnosticExecutor, AgnosticExecutorBuilder, AgnosticExecutorManager,
    new_agnostic_executor, get_global_executor, spawn, spawn_blocking//, spawn_local
};

#[ cfg(feature = "time") ]
mod time;

#[ cfg(feature = "time") ]
pub use time::*;