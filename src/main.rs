#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ParseResponse {
    parse: DisplayTitle,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct DisplayTitle {
    title: String,
    pageid: u64,
    displaytitle: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://en.wikipedia.org/w/api.php\
               ?action=parse\
               &format=json\
               &page=Rust_(programming_language)\
               &prop=displaytitle\
               &formatversion=2";
    let resp = reqwest::get(url)
        .await?
        .json::<ParseResponse>()
        .await?;
    println!("{resp:#?}");
    Ok(())
}
