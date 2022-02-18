use axum::extract::Extension;
use axum::response::Json;
use axum::routing::get;
use axum::AddExtensionLayer;
use axum::Router;
use clap::Parser;
use lindera::tokenizer::Tokenizer;
use lindera::tokenizer::TokenizerConfig;
use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Host
    #[clap(short, long)]
    host: String,

    /// Port
    #[clap(short, long)]
    port: u16,
}

#[derive(Serialize, Deserialize)]
struct Result {
    tokens: Vec<String>,
}

type SharedTokenizer = Arc<Mutex<Tokenizer>>;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let tokenizer = Arc::new(Mutex::new(
        Tokenizer::with_config(TokenizerConfig::default()).unwrap(),
    ));

    let args = Args::parse();

    let host = args.host;
    let port = args.port;
    let ip = IpAddr::from_str(host.as_str()).unwrap();
    let addr = SocketAddr::new(ip, port);

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .layer(AddExtensionLayer::new(tokenizer));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root(Extension(tokenizer): Extension<SharedTokenizer>) -> Json<Result> {
    let mut tokenizer_obj = tokenizer.lock().await;
    let tokens = tokenizer_obj.tokenize("すもももももももものうち").unwrap();

    let mut texts = Vec::new();
    for token in tokens {
        let text = token.text.to_string();
        texts.push(text);
    }

    Json(Result { tokens: texts })
}
