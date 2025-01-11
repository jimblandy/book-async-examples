use anyhow::{Result, Context};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    let traversal = Arc::new(Traversal::new());

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

    let start = "Rust (programming language)".to_string();
    Arc::clone(&traversal).visit(start.clone(), 2).await?;
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

/// A traversal of Wikipedia, starting from a given page.
struct Traversal {
    /// Titles of pages we have already visited.
    seen: Mutex<HashSet<String>>,

    /// Errors we've encountered when sending queries to Wikipedia.
    errors: Mutex<Vec<anyhow::Error>>,

    /// The time we're next allowed to send a request.
    next_turn: Mutex<Option<Instant>>,
}

impl Traversal {
    fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            errors: Mutex::new(Vec::new()),
            next_turn: Mutex::new(None),
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
        let subtasks = page_links(&page)
            .await?
            .into_iter()
            .map(|outgoing_link| {
                Arc::clone(&self).spawn_visit(outgoing_link, depth - 1)
            })
            .collect::<Vec<_>>();
        // All subtasks are running concurrently at this point.
        for subtask in subtasks {
            match subtask.await {
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
    ) -> tokio::task::JoinHandle<Result<()>> {
        tokio::task::spawn(self.visit(page, depth))
    }

    async fn save_error(&self, error: anyhow::Error) {
        self.errors.lock().await.push(error);        
    }

    async fn wait_for_turn(&self) {
        // https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
        const SPACING: Duration = Duration::from_millis(100);

        let now = Instant::now();

        let mut guard = self.next_turn.lock().await;
        let Some(ref mut next_turn) = *guard else {
            *guard = Some(now + SPACING);
            return;
        };

        let release_time = *next_turn;
        *next_turn = std::cmp::max(*next_turn, now) + SPACING;
        drop(guard); // don't hold the lock while we wait
        tokio::time::sleep_until(release_time).await;
    }
}


#[derive(Deserialize)]
enum Response {
    #[serde(rename = "parse")]
    Parse(ParseResponse),
    #[serde(rename = "error")]
    Error(ErrorResponse),
}

#[derive(Deserialize)]
struct ParseResponse {
    links: Vec<Link>,
}

#[derive(Deserialize)]
struct Link {
    ns: u32,
    exists: Option<String>,
    #[serde(rename = "*")]
    title: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    info: String,
}

async fn page_links(page: &str) -> Result<Vec<String>> {
    let url = format!("http://en.wikipedia.org/w/api.php\
                       ?action=parse&format=json&prop=links\
                       &page={page}"
    );
    eprintln!("Query: {url}");
    let response_body = reqwest::get(url)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let parsed = serde_json::from_str(&response_body)
        .with_context(|| format!("Failed to parse response:\n{response_body:?}"))?;
    match parsed {
        Response::Error(ErrorResponse { info }) => {
            anyhow::bail!("{info}")
        }
        Response::Parse(parse) => Ok(parse
            .links
            .into_iter()
            .filter(|link| link.ns == 0 && link.exists.is_some())
            .map(|link| link.title)
            .collect()),
    }
}
