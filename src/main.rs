#[derive(Debug, serde::Deserialize)]
struct Response {
    parse: ParseResponse,
}

#[derive(Debug, serde::Deserialize)]
struct ParseResponse {
    sections: Vec<Section>
}

#[derive(Debug, serde::Deserialize)]
struct Section {
    line: String,
    number: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://en.wikipedia.org/w/api.php\
               ?action=parse&format=json&prop=sections\
               &page=Rust_(programming_language)";
    let response = reqwest::get(url)
        .await?
        .json::<Response>()
        .await?;
    for section in &response.parse.sections {
        println!("{:6} {}", section.number, section.line);
    }
    Ok(())
}
