use reqwest::Result;

async fn get_text(url: &str) -> Result<String> {
    let body = reqwest::get(url)
        .await?
        .text()
        .await?;
    Ok(body)
}


#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://en.wikipedia.org/w/api.php\
               ?action=parse&format=json&prop=sections\
               &page=Rust_(programming_language)";
    let body = get_text(url).await?;
    println!("{body}");
    Ok(())
}
