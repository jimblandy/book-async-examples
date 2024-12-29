use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let traversal = Arc::new(Traversal::new());

    let start = "Rust (programming language)".to_string();
    Arc::clone(&traversal).visit(start.clone(), 3).await?;
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
}

impl Traversal {
    fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            errors: Mutex::new(Vec::new()),
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
    let url = format!(
        "http://localhost:3000/w/api.php\
                       ?action=parse&format=json&prop=links\
                       &page={page}"
    );
    eprintln!("Query: {url}");
    let response = reqwest::get(url)
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?;
    match response {
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
