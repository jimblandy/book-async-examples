use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let seen = Arc::new(Mutex::new(HashSet::new()));
    let start = "Rust (programming language)".to_string();

    add_reachable(start.clone(), Arc::clone(&seen), 2).await?;

    let arc_contents = Arc::into_inner(seen).unwrap();
    let mutex_contents = arc_contents.into_inner();
    let mut pages = Vec::from_iter(mutex_contents);
    pages.sort();

    println!("Pages reachable from {start}:");
    for page in pages {
        println!("  {page}");
    }

    Ok(())
}

async fn add_reachable(
    page: String,
    seen: Arc<Mutex<HashSet<String>>>,
    depth: usize,
) -> Result<()> {
    if !seen.lock().await.insert(page.clone()) {
        return Ok(());
    }

    if depth == 0 {
        return Ok(());
    }

    let links = page_links(&page).await?;
    let subtasks = links
        .into_iter()
        .map(|outgoing_link| {
            let seen = Arc::clone(&seen);
            spawn_add_reachable(outgoing_link, seen, depth - 1)
        })
        .collect::<Vec<_>>();

    for subtask in subtasks {
        subtask.await??;
    }

    Ok(())
}

fn spawn_add_reachable(
    page: String,
    seen: Arc<Mutex<HashSet<String>>>,
    depth: usize,
) -> tokio::task::JoinHandle<Result<()>> {
    tokio::task::spawn(add_reachable(page, seen, depth))
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
    let response: Response = reqwest::get(url).await?.json().await?;
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
