use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Deserialize)]
pub enum Response {
    #[serde(rename = "parse")]
    Parse(ParseResponse),
    #[serde(rename = "error")]
    Error(ErrorResponse),
}

#[derive(Deserialize)]
pub struct ParseResponse {
    pub links: Vec<Link>,
}

#[derive(Deserialize)]
pub struct Link {
    pub ns: u32,
    pub exists: Option<String>,
    #[serde(rename = "*")]
    pub title: String,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub info: String,
}

pub async fn page_links(page: &str) -> Result<Vec<String>> {
    //let url = format!("http://localhost:3000/w/api.php\
    let url = format!("http://en.wikipedia.org/w/api.php\
                       ?action=parse&format=json&prop=links\
                       &page={page}"
    );
    eprintln!("Query: {url}");
    let response_body = reqwest::get(url)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let parsed = serde_json::from_str(&response_body)
        .with_context(|| format!("Failed to parse response:\n{response_body:?}"))?;
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
