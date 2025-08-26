use wikipedia_reachable::links::page_links;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Some(title) = std::env::args().nth(1) else {
        anyhow::bail!("Usage: list-links TITLE");
    };

    for link in page_links(&title).await? {
        println!("{link}");
    }

    Ok(())
}
