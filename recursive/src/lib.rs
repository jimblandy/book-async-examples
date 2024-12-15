use std::future::Future;

pub async fn recur(depth: usize) {
    if depth == 0 {
        return;
    }
    spawn_recur(depth - 1);
}

fn spawn_recur(depth: usize) {
    spawn(recur(depth))
}

pub fn spawn(_future: impl Future + Send + 'static) { }
