#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://en.wikipedia.org/w/api.php\
               ?action=parse&format=json&prop=sections\
               &page=Rust_(programming_language)";
    let response = reqwest::get(url)
        .await?
        .text()
        .await?;
    println!("{response}");
    Ok(())
}
