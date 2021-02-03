The idea is to remove spawn_local from agnostic executor, and replace it with an alternative start_with_local that gives you a local executor on the start function that is not Send.

There you have the local and normal executors. If you need to you can implement a trick like the one shown in tokio where you use a channel to receive tasks that are executed locally.
In this case you would probably run start_with_local in another thread.

For libraries that's it, if it's not enough for you what start_with_local can give you, you should probably base your library on async_std instead and use it's compatibility layers if needed.

If the library needs the local executor, it should have a way to provide both executors to it, there are many ways to do it.

For final apps, if you choose an executor that supports it spawn_local will be available in the agnostic executor (async_std, wasm, futures ST, tokio ST, smol?, async_global_executor? async_std with tokio support?).
They should enable (depend on) a spawn_local feature that can be used with #[cfg(feature = "spawn_local")].

For final apps you should only enable the features of a single executor unless you are in a really unlikely and special situation where you need multiple different executors.
In this case, it's important that you really know what you are doing because spawn_local might be enabled on executors that don't support it and panic at runtime.

Please not that libraries should not enable any concrete executor features (except for dev deps), otherwise they won't be agnostic.

If they want to be partially agnostic, because they need spawn_local, or other future features (io?) they can gate the main library functions/structs with partially agnostic features like spawn_local.

------------------------------------

- Support spawn_local in smol using the ideas from async_global_executor
- Support async_global_executor (including spawn_local)
- Support tokio single threaded with spawn_local support
- Support futures single threaded with spawn_local support
- Support async_std/smol/... with tokio support (including spawn_local)
- Use futures-timer to try to fix the wasm issue (also for the futures executors)
- Find a better way for the tests that doesn't require cargo test -- --nocapture (do less in the common module? use something else?)
- Get our own macros for main, test, benchmark, ... or recommend using the upstream ones

-------------------------------------

When we release something useful move this TODO to github issues, or convert it into a roadmap where we integrate accepted issues to not depend on github issues.