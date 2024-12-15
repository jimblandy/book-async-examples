use std::future::Future;

pub async fn recur(depth: usize) {
    if depth == 0 {
        return;
    }
    spawn_recur(depth - 1);
}

// Inlining this function into `recur` causes compilation to fail:
// https://github.com/rust-lang/rust/issues/134101
fn spawn_recur(depth: usize) {
    spawn(recur(depth))
}

pub fn spawn(_future: impl Future + Send + 'static) { }
