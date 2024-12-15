# Test case for almost-recursive async functions

This program is the test case for [rust-lang/rustc#134101].

The code here compiles in Rust 1.83, but not if the body of
`spawn_recur` is inlined into `recur`. In that state, compiling with
`-Znext-solver` does work:

    RUSTFLAGS=-Znext-solver cargo +nightly check

[rust-lang/rustc#134101]: https://github.com/rust-lang/rust/issues/134101
