use anyhow::Result;
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
async fn main() -> Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    let args: AsyncTreeSitter = argh::from_env();

    log::info!("Serving async treesitter API at {:?}", args.address);

    warp::serve(warp::post()
                .and(warp::path!("v1" / "parse"))
                .and(warp::body::content_length_limit(100 * 1024))
                .and(warp::body::json::<ParseRust>())
                .map(handle_parse_rust))
        .run(args.address)
        .await;

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
struct ParseRust {
    source: String,
}

fn handle_parse_rust(parse_rust: ParseRust) -> http::Result<http::Response<String>> {
    log::trace!("handle_parse_rust {parse_rust:?}");
    http::Response::builder()
        .body(format!("yo dawg: {parse_rust:?}\n"))
}
