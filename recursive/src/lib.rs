use std::future::Future;

pub async fn recur(depth: usize) {
    if depth == 0 {
        return;
    }
    spawn(recur(depth - 1));
}

pub fn spawn(_future: impl Future + Send + 'static) { }
