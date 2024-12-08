use anyhow::Result;
use serde::Deserialize;

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
enum Response {
    parse(ParseResponse),
    error(ErrorResponse),
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
    let url = format!("https://en.wikipedia.org/w/api.php\
                       ?action=parse&format=json&prop=links\
                       &page={page}");
    let response: Response = reqwest::get(url)
        .await?
        .json()
        .await?;
    match response {
        Response::Error(ErrorResponse { info }) => {
            anyhow::bail!("{info}")
        },
        Response::Parse(parse) => Ok(
            parse.links.into_iter()
                .filter(|link| link.ns == 0 && link.exists.is_some())
                .map(|link| link.title)
                .collect()
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let response = page_links("splorf").await?;
    println!("{response:#?}");
    Ok(())
}
