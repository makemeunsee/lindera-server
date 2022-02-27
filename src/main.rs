use axum::body::Bytes;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::ContentLengthLimit;
use axum::extract::Extension;
use axum::response::Json;
use axum::routing::post;
use axum::AddExtensionLayer;
use axum::Router;
use clap::{AppSettings, Parser};
use serde_derive::{Deserialize, Serialize};
use tracing::{info, error};

use lindera::tokenizer::{Tokenizer, TokenizerConfig, UserDictionaryType};
use lindera_core::error::LinderaErrorKind;
use lindera_core::viterbi::{Mode, Penalty};
use lindera_core::LinderaResult;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None, setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    /// Host address
    #[clap(
        short = 'H',
        long = "host",
        default_value = "0.0.0.0",
        value_name = "HOST"
    )]
    host: String,

    /// HTTP port
    #[clap(
        short = 'p',
        long = "port",
        default_value = "8080",
        value_name = "PORT"
    )]
    port: u16,

    /// The dictionary direcory. If not specified, use the default dictionary.
    #[clap(short = 'd', long = "dict", value_name = "DICT")]
    dict: Option<PathBuf>,

    /// The user dictionary file path.
    #[clap(short = 'D', long = "user-dict", value_name = "USER_DICT")]
    user_dict: Option<PathBuf>,

    /// The user dictionary type. csv or bin
    #[clap(short = 't', long = "user-dict-type", value_name = "USER_DICT_TYPE")]
    user_dict_type: Option<String>,

    /// The tokenization mode. normal or search can be specified. If not specified, use the default mode.
    #[clap(short = 'm', long = "mode", value_name = "MODE")]
    mode: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Result {
    tokens: Vec<String>,
}

#[tokio::main]
async fn main() -> LinderaResult<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // make tokenizer config
    let mut config = TokenizerConfig {
        dict_path: args.dict,
        user_dict_path: args.user_dict,
        user_dict_type: UserDictionaryType::CSV,
        mode: Mode::Normal,
    };

    // set user dictionary type
    match args.user_dict_type {
        Some(ref user_dict_type) => {
            if user_dict_type == "csv" {
                config.user_dict_type = UserDictionaryType::CSV;
            } else if user_dict_type == "bin" {
                config.user_dict_type = UserDictionaryType::Binary;
            } else {
                return Err(LinderaErrorKind::Args.with_error(anyhow::anyhow!(
                    "Invalid user dictionary type: {}",
                    user_dict_type
                )));
            }
        }
        None => {
            config.user_dict_type = UserDictionaryType::CSV;
        }
    }

    // set tokenize mode
    match args.mode {
        Some(mode) => match mode.as_str() {
            "normal" => config.mode = Mode::Normal,
            "search" => config.mode = Mode::Decompose(Penalty::default()),
            "decompose" => config.mode = Mode::Decompose(Penalty::default()),
            _ => {
                return Err(LinderaErrorKind::Args
                    .with_error(anyhow::anyhow!("unsupported mode: {}", mode)))
            }
        },
        None => config.mode = Mode::Normal,
    }

    // make tokenizer
    let tokenizer = Tokenizer::with_config(config)?;

    let host = args.host;
    let port = args.port;
    let ip = IpAddr::from_str(host.as_str()).unwrap();
    let addr = SocketAddr::new(ip, port);

    // build our application with a route
    let app = Router::new()
        .route("/tokenize", post(tokenize))
        .layer(AddExtensionLayer::new(Arc::new(tokenizer)));

    // start the HTTP server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// basic handler that responds with a static string
async fn tokenize(
    ContentLengthLimit(bytes): ContentLengthLimit<Bytes, { 1024 * 5_000 }>,
    Extension(tokenizer): Extension<Arc<Tokenizer>>,
) -> Json<Result> {
    let text = match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(err) => {
            error!("{}", err);
            return Json(Result {
                tokens: vec![],
            })
        }
    };
    info!("text: {}", text);
    let tokens = match tokenizer.tokenize(text.as_str()) {
        Ok(tokens) => tokens,
        Err(err) => {
            error!("{}", err);
            return Json(Result {
                tokens: vec![],
            })
        }
    };

    let mut texts = Vec::new();
    for token in tokens {
        let text = token.text.to_string();
        texts.push(text);
    }

    Json(Result { tokens: texts })
}
