use anyhow::Result;
use wikipedia_reachable::links;

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
}

use std::sync::Arc;
use tokio::task::JoinSet;

impl Traversal {
    /// Visit all pages reachable from `page` within `depth` links.
    async fn visit(self: Arc<Self>, page: String, depth: usize) {
        if !self.seen.lock().unwrap().insert(page.clone()) {
            return;
        }

        if depth == 0 {
            return;
        }

        let links = match links::page_links(&page).await {
            Ok(links) => links,
            Err(error) => {
                self.errors.lock().unwrap().push(error);
                return;
            }
        };

        let mut join_set = JoinSet::new();
        for link in links {
            Arc::clone(&self).spawn_visit(link, depth - 1, &mut join_set);
        }
        join_set.join_all().await;
    }

    fn spawn_visit(self: Arc<Self>, page: String, depth: usize, join_set: &mut JoinSet<()>) {
        join_set.spawn(self.visit(page, depth));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let traversal = Arc::new(Traversal::new());

    let start = "Rust (programming language)".to_string();
    Arc::clone(&traversal).visit(start, 2).await;

    let traversal = Arc::into_inner(traversal).unwrap();
    let seen = traversal.seen.into_inner().unwrap();

    let mut sorted = Vec::from_iter(seen.into_iter());
    sorted.sort();
    for page in sorted {
        println!("{page}");
    }

    let errors = traversal.errors.into_inner().unwrap();
    for error in &errors {
        eprintln!("{error}");
    }
    if !errors.is_empty() {
        anyhow::bail!("Errors occurred during traversal");
    }

    Ok(())
}
