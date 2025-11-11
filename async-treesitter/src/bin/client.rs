use std::path::PathBuf;
use std::{net, fs};

/// Client to talk to the async-treesitter server.
#[derive(argh::FromArgs)]
struct Options {
    #[argh(option, default = r#"arg_address("0.0.0.0:3000")"#)]
    /// address to listen for HTTP requests on. (Default: 0.0.0.0:3000)
    address: net::SocketAddr,

    /// rust file to parse
    #[argh(positional)]
    filename: PathBuf,
}

fn arg_address(arg: &str) -> net::SocketAddr {
    arg.parse().unwrap()
}

#[derive(serde::Serialize, Debug)]
struct ParseRust {
    source: String,
    timeout_seconds: f32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Options = argh::from_env();

    let client = reqwest::Client::new();
    let request = ParseRust {
        source: fs::read_to_string(&args.filename)?,
        timeout_seconds: 10.0,
    };
    let response = client.post(format!("http://{}/v1/parse", args.address))
        .json(&request)
        .send()
        .await?;

    let parsed = response.error_for_status()?.text().await?;
    println!("{parsed}");
    
    Ok(())
}
