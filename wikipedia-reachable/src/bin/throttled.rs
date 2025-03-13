use wikipedia_reachable::links;

use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

/// A traversal of Wikipedia, starting from a given page.
struct Traversal {
    /// Titles of pages we have already visited.
    seen: Mutex<HashSet<String>>,

    /// Errors we've encountered when sending queries to Wikipedia.
    errors: Mutex<Vec<anyhow::Error>>,

    /// The time we're next allowed to send a request.
    next_turn: Mutex<Instant>,
}

impl Traversal {
    fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            errors: Mutex::new(Vec::new()),
            next_turn: Mutex::new(Instant::now()),
        }
    }

    /// Visit all pages reachable from `page` within `depth` links.
    async fn visit(
        self: Arc<Self>,
        page: String,
        depth: usize,
    ) -> Result<()> {
        if !self.seen.lock().await.insert(page.clone()) {
            return Ok(());
        }

        if depth == 0 {
            return Ok(());
        }

        self.wait_for_turn().await;

        let mut join_set = tokio::task::JoinSet::new();
        for outgoing_link in links::page_links(&page).await? {
            Arc::clone(&self).spawn_visit(outgoing_link, depth - 1, &mut join_set);
        }

        // All subtasks are running concurrently at this point.
        while let Some(subtask_result) = join_set.join_next().await {
            match subtask_result {
                Err(join_error) => self.save_error(join_error.into()).await,
                Ok(Err(visit_error)) =>  self.save_error(visit_error).await,
                Ok(Ok(())) => {}
            }
        }            

        Ok(())
    }

    fn spawn_visit(
        self: Arc<Self>,
        page: String,
        depth: usize,
        join_set: &mut tokio::task::JoinSet<Result<()>>,
    ) {
        join_set.spawn(self.visit(page, depth));
    }

    async fn save_error(&self, error: anyhow::Error) {
        self.errors.lock().await.push(error);        
    }

    async fn wait_for_turn(&self) {
        // https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
        const SPACING: Duration = Duration::from_millis(100);

        let now = Instant::now();

        let mut next_turn = self.next_turn.lock().await;
        let my_turn = *next_turn;
        *next_turn = std::cmp::max(*next_turn, now) + SPACING;
        drop(next_turn); // don't hold the lock while we wait
        tokio::time::sleep_until(my_turn).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let traversal = Arc::new(Traversal::new());

    spawn_metrics();

    let start = "Rust (programming language)".to_string();
    Arc::clone(&traversal).visit(start.clone(), 4).await?;

    let traversal = Arc::into_inner(traversal).unwrap();
    let seen = traversal.seen.into_inner();
    let mut sorted = Vec::from_iter(seen.into_iter());
    sorted.sort();
    for page in sorted {
        println!("{page}");
    }

    let errors = traversal.errors.into_inner();
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{error}");
        }
        anyhow::bail!("Errors occurred during traversal");
    }

    Ok(())
}

#[allow(dead_code)]
fn spawn_metrics() {
    tokio::spawn(async {
        let handle = tokio::runtime::Handle::current();
        loop {
            let metrics = handle.metrics();
            eprintln!("active: {}  queue: {}",
                      metrics.num_alive_tasks(),
                      metrics.global_queue_depth());
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}
