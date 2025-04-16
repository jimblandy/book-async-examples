use serde::Deserialize;

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

use anyhow::{Context, Result};

pub async fn page_links(title: &str) -> Result<Vec<String>> {
    // To use the mock API server, change the below to:
    //const SERVER: &str = "http://localhost:3000/w/api.php";
    const SERVER: &str = "http://en.wikipedia.org/w/api.php";

    let url = format!("{SERVER}?action=parse&format=json&prop=links&page={title}");
    let parsed = reqwest::get(url)
        .await
        .context("Error sending request")?
        .error_for_status()
        .context("Error from server")?
        .json::<Response>()
        .await
        .context("Error parsing response")?;

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
