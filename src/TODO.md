- Add support fo IO with the async-net and  async-fs crates from smol (also see the async-compat and blocking crates async-lock)
    - Smol seems to be a great set of libs we can just re-export under some features (use compatible versions with smol executor and async std)
    - Think about what can be done in wasm
    - This doesn't use the underlying libraries in tokio. Maybe have a minimal set of io (under a minimal-io feature) that uses async-compat, is it worth it to avoid the extra dependencies?.
    - Users can always 
- Give access to the underlying executor in case we need some custom features
- Add spawn_local support directly on the main executor under a feature flag for the executors that can support it (async_std [also with tokio support], wasm, futures ST, tokio ST)
    - spawn_local with multiple executors enable might panic at runtime if used on an unsupported executor (see the block_on feature)
    - Libraries can require the spawn_local feature and be agnostic over a smaller set of executors
    - Support spawn_local in smol using the ideas from async_global_executor
- Support other executors and variants
    - Support async_global_executor (including spawn_local)
    - Support tokio single threaded with spawn_local support
    - Support futures single threaded with spawn_local support
    - Support async_std/smol/... with tokio support (including spawn_local)
- Get our own macros for main, test, benchmark, ... or recommend using the upstream ones
- Test helpers for specific runtime tests test_in_X other than wasm, also for the native_spawn_local subset to be able to test it (extract common code)
- Provide a dummy entry for the JoinHandle enum so that when no other features are enabled the type T is used, this will be disabled in any real use case
- Improve documentation
    - Add empty lines to create new paragraphs on the documentation
    - Write example crates
    - Try to hide macros from the root module, if it's even possible
- Use this idea, and maybe make all tests async, to unify all the tests, wasm and native:
    - https://github.com/wasm-rs/async-executor/blob/df48775b37bf62fbc1036a856151b526a3f700ab/src/single_threaded.rs#L288
    - Not clear how to do it !!!! we cannot convert native tests to async, because async tests are not supported
    - But we cannot remove async from wasm as we don't have blocking calls (in particular manager.start is not blocking)
- Write more tests
    - Test interval stream
- Allow  to clone the JoinHandle based on ideas from https://docs.rs/futures/0.3.18/futures/future/trait.FutureExt.html#method.shared
- Think if is possible to have access to the current executor without needed to pass it along
- Allow to get an independent cancel handle. 
- Support scoped async tasks that block on the scope, based on this idea:
    - https://github.com/rmanoka/async-scoped/blob/master/src/scoped.rs (simple to migrate implementation to this library, only allow the safe blocking, add disclaimer about recursion in async-std and more)
    - Presentation and safety discussion: https://www.reddit.com/r/rust/comments/ee3vsu/asyncscoped_spawn_non_static_futures_with_asyncstd/
    - The implementation is based on this, scoped should be under a feature to avoid extra dependencies: https://docs.rs/futures/0.3.18/futures/stream/struct.FuturesUnordered.html
    - If we just need to spawn a single task from sync code, maybe this could be added directly on block_on, but inside the async we cannot spawn any new tasks (as we would need the full implementation)
    - In this implementation the scope is send & sync (see docs), but  not clone  and requires &mut self for spawning. This has limitations.
    - Other ideas
        - https://github.com/tokio-rs/tokio/issues/1879 (closed for not finding a ood solution)
        - https://github.com/withoutboats/juliex/issues/19 (old comments about async-scoped)
        - https://github.com/rustasync/runtime/issues/55 (old comments about async-scoped)
    - Think if this is really needed, or if there are better alternatives. Because it's not perfect at all !!!!
- Add support for spawning long running tasks in a different executor/threadpool (see https://lib.rs/crates/futures-cputask for the idea, but we should use our own implementation)
    - See the blocking crate from smol