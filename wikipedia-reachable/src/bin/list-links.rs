use wikipedia_reachable::links::page_links;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Some(url) = std::env::args().nth(1) else {
        anyhow::bail!("Usage: list-links URL");
    };

    for link in page_links(&url).await? {
        println!("{link}");
    }

    Ok(())
}
