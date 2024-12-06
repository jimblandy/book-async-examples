use anyhow::Result;
use argh::FromArgs;
use warp::Filter as _;
use warp::http;
use std::net;
use std::str::FromStr as _;

#[derive(FromArgs)]
/// Serve a directory's contents, providing server-sent events when files are changed.
struct MockWiki {
    #[argh(option, default = r#"arg_address("0.0.0.0:3000")"#)]
    /// address to listen for HTTP requests on. (Default: 0.0.0.0:3000)
    address: net::SocketAddr,
}

fn arg_address(arg: &str) -> net::SocketAddr {
    net::SocketAddr::from_str(arg).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: MockWiki = argh::from_env();

    log::info!("Serving Mock Wikipedia API at {:?}", args.address);

    // let url = format!("https://en.wikipedia.org/w/api.php\
    //                    ?action=parse&format=json&prop=links\
    //                    &page={page}");
    
    warp::serve(warp::get()
                .and(warp::path!("w" / "api.php"))
                .and(warp::query::<Query>())
                .map(handle_query))
        .run(args.address)
        .await;

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Query {
    #[serde(rename = "parse")]
    Parse { 
        format: Format,
        prop: Prop,
        page: String,
    }
}

#[derive(serde::Deserialize, Debug)]
enum Format {
    #[serde(rename = "json")]
    Json,
}

#[derive(serde::Deserialize, Debug)]
enum Prop {
    #[serde(rename = "links")]
    Links,
}

fn handle_query(query: Query) -> http::Result<http::Response<String>> {
    log::trace!("handle_query {query:?}");
    match query {
        Query::Parse { format, prop, page } => parse(page, format, prop),
    }
}

#[derive(serde::Serialize, Debug)]
struct Answer {
    parse: ParseAnswer,
}

#[derive(serde::Serialize, Debug)]
struct ParseAnswer {
    title: String,
    pageid: u64,
    links: Vec<Link>,
}

#[derive(serde::Serialize, Debug)]
struct Link {
    ns: u64,
    exists: Option<String>,
    #[serde(rename = "*")]
    title: String,
}

fn parse(_page: String, _format: Format, _prop: Prop) -> http::Result<http::Response<String>> {
    let answer = Answer {
        parse: ParseAnswer {
            title: "Yahoo!".to_string(),
            pageid: 1729,
            links: vec![
                Link {
                    ns: 0,
                    exists: Some("".to_string()),
                    title: "Blort".to_string()
                }
            ],
        }
    };
    http::Response::builder()
        .status(http::StatusCode::OK)
        .body(serde_json::to_string(&answer).unwrap())
}

