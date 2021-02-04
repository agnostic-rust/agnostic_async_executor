//! Agnostic Async Executor
//! TODO Doc

#![deny(missing_docs)]

mod agnostic_executor;

pub use agnostic_executor::{
    JoinHandle, AgnosticExecutor, AgnosticExecutorBuilder, AgnosticExecutorManager,
    new_agnostic_executor, get_global_executor, spawn, spawn_blocking//, spawn_local
};

// TODO Pub the mod instead of the pub use below, like test
#[ cfg(feature = "time") ]
mod time;

#[ cfg(feature = "time") ]
pub use time::*;

#[ cfg(feature = "test") ]
pub mod test;