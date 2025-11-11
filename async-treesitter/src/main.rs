use argh::FromArgs;
use warp::Filter as _;
use warp::http;
use std::net;
use std::str::FromStr as _;

#[derive(FromArgs)]
/// Parse trees presented by clients as Rust.
struct AsyncTreeSitter {
    #[argh(option, default = r#"arg_address("0.0.0.0:3000")"#)]
    /// address to listen for HTTP requests on. (Default: 0.0.0.0:3000)
    address: net::SocketAddr,
}

fn arg_address(arg: &str) -> net::SocketAddr {
    net::SocketAddr::from_str(arg).unwrap()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    let args: AsyncTreeSitter = argh::from_env();

    log::info!("Serving async treesitter API at {:?}", args.address);

    warp::serve(warp::post()
                .and(warp::path!("v1" / "parse"))
                .and(warp::body::content_length_limit(100 * 1024))
                .and(warp::body::json::<ParseRust>())
                .map(parse_rust_request))
        .run(args.address)
        .await;

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
struct ParseRust {
    source: String,
    timeout_seconds: f32,
}

fn parse_rust_request(request: ParseRust) -> http::Result<http::Response<String>> {
    log::trace!("handle_parse_rust {request:?}");

    let builder = http::Response::builder();
    match parse_rust(request) {
        Ok(tree) => {
            let mut tree = tree.root_node().to_sexp();
            tree.push('\n');
            builder
                .header("Content-Type", "application/json")
                .body(tree)
        }
        Err(error) => {
            builder
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(error.to_string())
        }
    }
}

fn parse_rust(request: ParseRust) -> anyhow::Result<tree_sitter::Tree> {
    let timeout = std::time::Duration::from_secs_f32(request.timeout_seconds);
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    let Some(tree) = parser.parse(&request.source, None) else {
        anyhow::bail!("request timed out after {timeout:?} seconds");
    };

    Ok(tree)
}
