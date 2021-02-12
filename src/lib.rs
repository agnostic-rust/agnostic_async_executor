//! # Agnostic Async Executor
//! The agnostic_async_executor crate is a rust library that helps you write async libraries and applications that are executor agnostic.
//! 
//! It supports the most common executors for a wide variety of use cases. Tokio and Async Std when you need power and compatibility, Smol and the Futures crate executors when you need something lightweight, and Wasm Bindgen if you need wasm support.
//!
//! It is really simple to write agnostic async libraries that really contribute to the whole ecosystem without locking your users to a single executor. Just depend on this library without any executor specific features and ask for an AgnosticExecutor to be provided by your users. You can see a complete example under example_crates on the source code.
//!
//! ## Features
//!
//! The main purpose of this library is to give you an AgnosticExecutor type that is easy to work with. It is a plain struct that implements Clone, Send and can be easily stored and shared as needed.
//!
//! The AgnosticExecutor can be used to spawn new async tasks, and you have the option to spawn potentially blocking tasks (but be careful as this is just a hint to the executor that it might be ignored).
//!
//! If you need to execute tasks that are not Send, you can do it using the LocalAgnosticExecutor, that is only available on the thread that starts the executor. In the future we will add support for spawning local tasks from inside non-local tasks under a feature flag for the underlying executors that support it, but be careful because this limits the choice of executors available, in particular it will not support tokio.
//!
//! Apart from executing tasks this library provides an agnostic way to deal with async time. It provides you with a way to sleep the current async task for a given duration, add a timeout to an async task, wait for a repeated interval, and measure time with a stopwatch. All of this working the underlying methods of each executor when available for best performance and accuracy while working on every executor and platform, including wasm.
//!
//! Another important feature is testing, and we provide an agnostic way to test your async code in all the supported executors without writing independent tests for each one of them and with a uniform testing strategy that works even in the most challenging cases like nested async calls in  wasm.
//!
//! Finally, more features and maybe a whole ecosystem of crates is being worked out around agnostic_async_executor. For example, a lightweight but powerful actor system is being implemented right now for those times when you need async code but you also need permanent entities with mutable access to it's own state.
//!
//!
//! ### Supported cargo features
//!
//! TODO
//!
//! ## How to use
//!
//! ### In a library
//!
//! Add `agnostic_async_executor = "0.2.0"` to your dependencies. You can include generic features, but not executor dependent features  if you want it to be agnostic.
//!
//! ```text
//!     TODO
//! ```
//!
//! ### In an application that uses an agnostic library
//!
//! Add `agnostic_async_executor = { version = "0.2.0", features = ["xxx_executor"]}` to your dependencies. You need to include at least an executor feature to be able to create the executor and pass it to the library.
//!
//! ```text
//!     TODO
//! ```
//! ## Why should you use this library
//!
//! Other agnostic executor libraries already exist, but they all have many drawbacks that avoid using them as the backbone of an executor agnostic async ecosystem.
//!
//! Other libraries are mostly based on traits that are implemented for the different executors. In theory this is a good idea because you could just implement the trait for any executor you need. 
//! 
//! But in practice this is not as simple as it seems, and even when possible, it's more convenient to just integrate the new executor implementation in the library itself. So we are not winning much from using a trait.
//!
//! And the problem of using traits is that they are really limiting in rust. 
//!
//! For example, the [async_executors](https://github.com/najamelan/async_executors) crate gives you an object-safe trait that is pretty convenient, but in order to do that you lose all your flexibility as you need to provide the return type of any spawn function in advance because it is a parameter of the trait itself. This might be possible for some libraries but in other cases is a severe limitation. And the way of working with it is very different from working directly with the underlying executors.
//!
//! The alternative we find it in the [agnostik](https://github.com/bastion-rs/agnostik) crate. They decided to go with an easy and powerful api, but at the cost of the trait not being object-safe. This might seem a small detail, but it highly impacts the rest of your application. To store an executor anywhere you need to have a concrete type, and in other to do that you need to add a generic parameter for it all around your library. 
//!
//! The futures crate also provides some traits to make executor agnostic libraries but they don't provide implementations and are really limited in their own ways.
//!
//! For more details on the problems of other crates you can see an explanation for the need of a library like this on this [comment](https://github.com/riker-rs/riker/pull/152#issuecomment-772747030)
//!
//! The solution to all of this is to forget about having an agnostic executor trait and have an AgnosticExecutor struct instead. This struct is simple to use, without any of the previous limitations, and it can be stored, cloned, send across threads as you need.
//!
//! All the implementation details are hidden and you just can use it as you would use the underlying executors. Powerful but simple.
//!
//! The only drawback is that you cannot implement support for new executors outside of the ones provided by library, so if you need support for a new executor you can create a pull request on github. But as we said before, in practice this is what you would do,or even had to do with the trait based alternatives.
//!
//! ## License
//! This project is licensed under either of
//!
//! - Apache License, Version 2.0, ([link](https://www.apache.org/licenses/LICENSE-2.0))
//! - MIT license ([link](https://opensource.org/licenses/MIT))
//! at your option.
//!
//! This project was originally forked from [async-spawner](https://github.com/dvc94ch/async-spawner) from David Craven <david@craven.ch> but it has been redesigned to be a completely different library. Still, it's possible that some of the original code remains in some form. 
//!

#![deny(missing_docs)]

mod executors;

pub use executors::{
    JoinHandle, AgnosticExecutor, AgnosticExecutorBuilder, AgnosticExecutorManager,
    new_agnostic_executor, get_global_executor, spawn, spawn_blocking
};

#[ cfg(feature = "time") ]
pub mod time;

#[ cfg(feature = "test") ]
pub mod test;