use wikipedia_reachable::links;
use anyhow::Result;

use std::collections::HashSet;
use std::sync::Mutex;

/// A traversal of Wikipedia, starting from a given page.
struct Traversal {
    /// Titles of pages we have already visited.
    seen: Mutex<HashSet<String>>,

    /// Errors we've encountered when sending queries to Wikipedia.
    errors: Mutex<Vec<anyhow::Error>>,
}

impl Traversal {
    fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            errors: Mutex::new(Vec::new()),
        }
    }

    async fn save_error(&self, error: anyhow::Error) {
        self.errors.lock().unwrap().push(error);
    }
}

use std::sync::Arc;

impl Traversal {
    fn spawn_visit(
        self: Arc<Self>,
        page: String,
        depth: usize,
        join_set: &mut tokio::task::JoinSet<Result<()>>,
    ) {
        join_set.spawn(self.visit(page, depth));
    }
}    

impl Traversal {
    /// Visit all pages reachable from `page` within `depth` links.
    async fn visit(self: Arc<Self>, page: String, depth: usize) -> Result<()> {
        if !self.seen.lock().unwrap().insert(page.clone()) {
            return Ok(());
        }

        if depth == 0 {
            return Ok(());
        }

        let mut join_set = tokio::task::JoinSet::new();
        for outgoing_link in links::page_links(&page).await? {
            let new_self = Arc::clone(&self);
            new_self.spawn_visit(outgoing_link, depth - 1, &mut join_set);
        }

        // All subtasks are in progress at this point, but almost all
        // are probably blocked waiting for a network response.

        while let Some(subtask_result) = join_set.join_next().await {
            match subtask_result {
                Err(join_error) => self.save_error(join_error.into()).await,
                Ok(Err(visit_error)) => self.save_error(visit_error).await,
                Ok(Ok(())) => {}
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let traversal = Arc::new(Traversal::new());

    let start = "Rust (programming language)".to_string();
    Arc::clone(&traversal).visit(start.clone(), 2).await?;

    let traversal = Arc::into_inner(traversal).unwrap();
    let seen = traversal.seen.into_inner().unwrap();
    let mut sorted = Vec::from_iter(seen.into_iter());
    sorted.sort();
    for page in sorted {
        println!("{page}");
    }

    let errors = traversal.errors.into_inner().unwrap();
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{error}");
        }
        anyhow::bail!("Errors occurred during traversal");
    }

    Ok(())
}
