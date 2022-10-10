use axum::body::Bytes;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{ContentLengthLimit, Extension};
use axum::response::Json;
use axum::routing::post;
use axum::Router;
use clap::Parser;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

use lindera::error::LinderaErrorKind;
use lindera::mode::{Mode, Penalty};
use lindera::tokenizer::{DictionaryKind, Tokenizer, TokenizerConfig};
use lindera::tokenizer::{DEFAULT_DICTIONARY_KIND, SUPPORTED_DICTIONARY_KIND};
use lindera::LinderaResult;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Host address
    #[clap(
        short = 'H',
        long = "host",
        default_value = "0.0.0.0",
        value_name = "HOST",
        display_order = 1
    )]
    host: String,

    /// HTTP port
    #[clap(
        short = 'p',
        long = "port",
        default_value = "8080",
        value_name = "PORT",
        display_order = 2
    )]
    port: u16,

    /// The dictionary type. local and ipadic are available.
    #[clap(
        short = 't',
        long = "dict-type",
        value_name = "DICT_TYPE",
        default_value = DEFAULT_DICTIONARY_KIND,
        display_order = 3
    )]
    dict_type: String,

    /// The dictionary directory. Specify the directory path of the dictionary when "local" is specified in the dictionary type.
    #[clap(short = 'd', long = "dict", value_name = "DICT", display_order = 4)]
    dict: Option<PathBuf>,

    /// The tokenization mode. normal, search and decompose are available.
    #[clap(
        short = 'm',
        long = "mode",
        value_name = "MODE",
        default_value = "normal",
        display_order = 5
    )]
    mode: String,
}

#[tokio::main]
async fn main() -> LinderaResult<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut config = TokenizerConfig::default();

    // dictionary type
    match args.dict_type.as_str() {
        #[cfg(feature = "ipadic")]
        "ipadic" => {
            config.dictionary.kind = DictionaryKind::IPADIC;
        }
        #[cfg(feature = "unidic")]
        "unidic" => {
            config.dictionary.kind = DictionaryKind::UniDic;
        }
        #[cfg(feature = "ko-dic")]
        "ko-dic" => {
            config.dictionary.kind = DictionaryKind::KoDic;
        }
        #[cfg(feature = "cc-cedict")]
        "cc-cedict" => {
            config.dictionary.kind = DictionaryKind::CcCedict;
        }
        _ => {
            return Err(LinderaErrorKind::Args.with_error(anyhow::anyhow!(format!(
                "{:?} are available for --dict-type",
                SUPPORTED_DICTIONARY_KIND
            ))));
        }
    }

    // mode
    match args.mode.as_str() {
        "normal" => config.mode = Mode::Normal,
        "search" => config.mode = Mode::Decompose(Penalty::default()),
        "decompose" => config.mode = Mode::Decompose(Penalty::default()),
        _ => {
            return Err(LinderaErrorKind::Args
                .with_error(anyhow::anyhow!("unsupported mode: {}", args.mode)));
        }
    }

    // create tokenizer
    let tokenizer = Tokenizer::with_config(config)?;

    let host = args.host;
    let port = args.port;
    let ip = IpAddr::from_str(host.as_str()).unwrap();
    let addr = SocketAddr::new(ip, port);

    // build our application with a route
    let app = Router::new()
        .route("/tokenize", post(tokenize))
        .layer(Extension(Arc::new(tokenizer)));

    // start the HTTP server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct DetailedResult {
    text: String,
    detail: Vec<String>,
}

// basic handler that responds with a static string
async fn tokenize(
    ContentLengthLimit(bytes): ContentLengthLimit<Bytes, { 1024 * 5_000 }>,
    Extension(tokenizer): Extension<Arc<Tokenizer>>,
) -> Json<Value> {
    let text = match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(err) => {
            error!("{}", err);
            let err_json = serde_json::json!({ "error": format!("{}", err) });
            return Json(err_json);
        }
    };
    info!("text: {}", text);

    // tokenize
    let tokens = match tokenizer.tokenize(text.as_str()) {
        Ok(tokens) => tokens,
        Err(err) => {
            error!("{}", err);
            let err_json = serde_json::json!({ "error": format!("{}", err) });
            return Json(err_json);
        }
    };

    let detailed_results: Vec<DetailedResult> = tokens
        .iter()
        .map(|token| DetailedResult {
            text: token.text.to_owned(),
            detail: tokenizer.word_detail(token.word_id).unwrap(),
        })
        .collect();

    Json(serde_json::value::to_value(&detailed_results).unwrap())
}
