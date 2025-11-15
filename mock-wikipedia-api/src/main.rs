use anyhow::Result;
use argh::FromArgs;
use axum::routing::get;
use axum::{Form, Router};
use tokio::net::TcpListener;
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
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    let args: MockWiki = argh::from_env();

    log::info!("Serving Mock Wikipedia API at {:?}", args.address);

    // let url = format!("https://en.wikipedia.org/w/api.php\
    //                    ?action=parse&format=json&prop=links\
    //                    &page={page}");
    
    let listener = TcpListener::bind(args.address).await?;
    let app = Router::new().route("/w/api.php", get(handle_query));
    log::info!("Serving async treesitter API at {:?}", args.address);
    axum::serve::serve(listener, app).await?;

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

async fn handle_query(Form(query): Form<Query>) -> http::Response<String> {
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

fn parse(page: String, _format: Format, _prop: Prop) -> http::Response<String> {
    if page == "Rust (programming language)-1-2" {
        // body() fails if some previous builder method was given invalid arguments,
        // but we are passing valid arguments, so we can unwrap()
        return http::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("Injected NOT_FOUND, for testing".to_string())
            .unwrap();
    }
    if page == "Rust (programming language)-2-1" {
        return http::Response::builder()
            .body(r#"{ "zloop": "murf" }"#.to_string())
            .unwrap();
    }

    fn link(title: String) -> Link {
        Link {
            ns: 0,
            exists: Some("".to_string()),
            title,
        }
    }
    let pageid = {
        use std::hash::Hash as _;
        use std::hash::Hasher as _;
        let mut s = std::hash::DefaultHasher::new();
        page.hash(&mut s);
        s.finish()
    };
    let answer = Answer {
        parse: ParseAnswer {
            title: page.clone(),
            pageid,
            links: (1..6)
                .map(|i| {
                    link(format!("{page}-{i}"))
                })
                .collect(),
        }
    };
    http::Response::builder()
        .body(serde_json::to_string(&answer).unwrap())
        .unwrap()
}

