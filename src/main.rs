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
use tracing::{error, info};
use serde_json::Value;

use lindera::error::LinderaErrorKind;
use lindera::formatter::format;
use lindera::formatter::Format;
use lindera::mode::{Mode, Penalty};
use lindera::tokenizer::{DictionaryType, Tokenizer, TokenizerConfig, UserDictionaryType};
use lindera::tokenizer::{DEFAULT_DICTIONARY_TYPE, SUPPORTED_DICTIONARY_TYPE};
use lindera::LinderaResult;

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

    /// The dictionary type. local and ipadic are available.
    #[clap(
        short = 't',
        long = "dict-type",
        value_name = "DICT_TYPE",
        default_value = DEFAULT_DICTIONARY_TYPE
    )]
    dict_type: String,

    /// The dictionary direcory. Specify the directory path of the dictionary when "local" is specified in the dictionary type.
    #[clap(short = 'd', long = "dict", value_name = "DICT")]
    dict: Option<PathBuf>,

    /// The user dictionary file path.
    #[clap(short = 'D', long = "user-dict", value_name = "USER_DICT")]
    user_dict: Option<PathBuf>,

    /// The user dictionary type. csv and bin are available.
    #[clap(short = 'T', long = "user-dict-type", value_name = "USER_DICT_TYPE")]
    user_dict_type: Option<String>,

    /// The tokenization mode. normal, search and decompose are available.
    #[clap(
        short = 'm',
        long = "mode",
        value_name = "MODE",
        default_value = "normal"
    )]
    mode: String,
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

    let mut config = TokenizerConfig::default();

    // dictionary type
    match args.dict_type.as_str() {
        #[cfg(feature = "ipadic")]
        "ipadic" => {
            config.dict_type = DictionaryType::Ipadic;
        }
        #[cfg(feature = "unidic")]
        "unidic" => {
            config.dict_type = DictionaryType::Unidic;
        }
        #[cfg(feature = "ko-dic")]
        "ko-dic" => {
            config.dict_type = DictionaryType::Kodic;
        }
        #[cfg(feature = "cc-cedict")]
        "cc-cedict" => {
            config.dict_type = DictionaryType::Cedict;
        }
        "local" => {
            config.dict_type = DictionaryType::LocalDictionary;
            config.dict_path = args.dict;
        }
        _ => {
            return Err(LinderaErrorKind::Args.with_error(anyhow::anyhow!(format!(
                "{:?} are available for --dict-type",
                SUPPORTED_DICTIONARY_TYPE
            ))));
        }
    }

    // user dictionary path
    config.user_dict_path = args.user_dict;

    // user dictionary type
    match args.user_dict_type {
        Some(ref user_dict_type) => match user_dict_type.as_str() {
            "csv" => config.user_dict_type = UserDictionaryType::Csv,
            "bin" => config.user_dict_type = UserDictionaryType::Binary,
            _ => {
                return Err(LinderaErrorKind::Args.with_error(anyhow::anyhow!(
                    "invalid user dictionary type: {}",
                    user_dict_type
                )))
            }
        },
        None => {
            config.user_dict_type = UserDictionaryType::Csv;
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

    // output result
    let output_json = match format(tokens, Format::Json) {
        Ok(output) => output,
        Err(err) => {
            error!("{}", err);
            let err_json = serde_json::json!({ "error": format!("{}", err) });
            return Json(err_json);
        }
    };

    let json_value = serde_json::from_str(&output_json).unwrap();

    Json(json_value)
}
