#[macro_use] extern crate log;
use std::path::PathBuf;
use std::str::FromStr;
use std::fs::File;
use polars::prelude::{DataFrameJoinOps, SortMultipleOptions};
use clap::{Parser, ValueEnum, builder::PossibleValue};
use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use evtx_clustering::evtx::EvtxHandler;
use evtx_clustering::errors::CustomError;
use evtx_clustering::filter::{Filter, FilterRule};
use evtx_clustering::embedding::EmbeddingsHandler;
use evtx_clustering::cluster::get_cluster_mapping;
use polars::prelude::{SerWriter, CsvWriter};
use openai_api_rs::v1::common::*;

static VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum EmbeddingModel {
    TextEmbedding3Small,
    TextEmbedding3Large,
    TextEmbeddingAda002,
}
impl ValueEnum for EmbeddingModel {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::TextEmbedding3Small, Self::TextEmbedding3Large, Self::TextEmbeddingAda002]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::TextEmbedding3Small => PossibleValue::new(TEXT_EMBEDDING_3_SMALL).help("Model"),
            Self::TextEmbedding3Large => PossibleValue::new(TEXT_EMBEDDING_3_LARGE).help("Model"),
            Self::TextEmbeddingAda002 => PossibleValue::new(TEXT_EMBEDDING_ADA_002).help("Model"),
        })
    }
}
impl std::fmt::Display for EmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
impl std::str::FromStr for EmbeddingModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}


/// A tool that can extract commands from EVTX files and summarize clusters.
/// Currently this tool only extracts commands that are found in the Event.EventData.CommandLine
/// attribute.
#[derive(Parser, Debug)]
#[command(
    author = "Matthew Seyer",
    version=VERSION,
)]
struct App {
    /// The source that contains EVTX records.
    #[arg(short, long, required=true)]
    source: PathBuf,
    /// The csv output file to write output to.
    #[arg(short, long, required=true)]
    csv_output: PathBuf,
    /// The embeddings cache directory.
    #[arg(short, long, required=true)]
    cache: PathBuf,
    /// OpenAI API token. If not used, the OPENAI_KEY env var will be used or an error will be thrown.
    #[arg(long, required=false)]
    openai_token: Option<String>,
    /// Embedding model selection.
    #[arg(long, required=false, default_value="text-embedding-3-small")]
    embedding_model: EmbeddingModel,
    /// Embedding model selection.
    #[arg(long, required=false)]
    embedding_dimensions: Option<i32>,
    /// Set the clustering tolerance threshold.
    #[arg(long, required=false, default_value="0.5")]
    cluster_tolerance: f32,
    /// Set the cluster grouping threshold.
    #[arg(long, required=false, default_value="2")]
    cluster_grouping: usize,
    /// The logging level to use.
    #[arg(long, required=false, default_value="Info", value_parser=["Off", "Error", "Warn", "Info", "Debug", "Trace"])]
    logging: String,
}
impl App {
    /// Set logging
    fn set_logging(&self) -> Result<(), CustomError> {
        let level = self.logging.as_str();

        let message_level = LevelFilter::from_str(level)
            .map_err(|e|
                CustomError::general_error(format!("Could not set logging level: {e:?}"))
            )?;

        let _ = Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(message_level)
            .level_for("evtx", log::LevelFilter::Error)
            .chain(std::io::stderr())
            .apply()
            .map_err(|e|
                CustomError::general_error(format!("Error initializing fern logging: {e:?}"))
            )?;
        
        // Ensure that logger was dispatched
        trace!("Logging has been initialized!");

        Ok(())
    }
}


#[tokio::main]
async fn main() {
    let app: App = App::parse();
    app.set_logging()
        .expect("Error setting logging!");

    let csv_output_location = app.csv_output.clone();
    let source_location = app.source.clone();
    let embedding_model = app.embedding_model.to_string();
    let embedding_dimensions = app.embedding_dimensions.clone();
    let cluster_tolerance = app.cluster_tolerance.clone();
    let cluster_grouping = app.cluster_grouping.clone();

    if !csv_output_location.parent().unwrap().exists() {
        std::fs::create_dir_all(&csv_output_location.parent().unwrap())
            .expect("Cannot creat output dir.");
    }

    // Create EVTX filter to pass to EvtxHandler
    let filter = Filter::OrFilter(vec![
        FilterRule::from_jmes("Event.EventData.CommandLine").unwrap()
    ]);

    // Create a EvtxHandler to perform EVTX opterations
    let evtx_handler = EvtxHandler::from_source(source_location)
        .with_filter(filter)
        .add_transformer_field_from_pattern("Timestamp", "Event.System.TimeCreated_attributes.SystemTime").unwrap()
        .add_transformer_field_from_pattern("Computer", "Event.System.Computer").unwrap()
        .add_transformer_field_from_pattern("Provider", "Event.System.Provider_attributes.Name").unwrap()
        .add_transformer_field_from_pattern("EventID", "Event.System.EventID").unwrap()
        .add_transformer_field_from_pattern("CommandLine", "Event.EventData.CommandLine").unwrap();

    // Fetch the OpenAI API key
    let api_key = match app.openai_token {
        Some(k) => k,
        None => std::env::var("OPENAI_KEY")
            .expect("No openai_token was provided and OPENAI_KEY env var is not set.")
    };
    // Create an EmbeddingsHandler to perform embedding tasks
    let embedding_handler = EmbeddingsHandler::new(
        api_key,
        embedding_model,
        embedding_dimensions,
        10
    ).with_cache(&app.cache)
        .expect("Error setting cache.");

    let df = evtx_handler.parse_into_dataframe()
        .expect("Error parsing evtx records into dataframe.");

    // Get all the CommandLine values
    let cmds: Vec<String> = df["CommandLine"]
        .unique_stable()
        .expect("Error computing unique values.")
        .str()
        .expect("Values are not strings.")
        .into_iter()
        .map(|v| {
            match v {
                Some(s) => s.to_string(),
                None => String::from("")
            }
        })
        .collect();

    // Get embeddings for command line values
    let value_embeddings = embedding_handler.get_embeddings(cmds)
        .await
        .expect("Error getting embeddings for commands.");

    let df_embeddings = get_cluster_mapping(
        value_embeddings,
        cluster_grouping,
        cluster_tolerance
    ).expect("Error getting clustered dataframe.");

    let mut df = df.left_join(
        &df_embeddings,
        &["CommandLine"],
        &["value"]
    ).expect("Error joining embeddings dataframe!");

    let mut output_csv_fh: File = File::create(csv_output_location).unwrap();
    CsvWriter::new(&mut output_csv_fh)
        .include_header(true)
        .finish(&mut df)
        .unwrap();

    println!("{}", df.sort(
        ["cluster"], SortMultipleOptions::new().with_order_descending(true)
    ).unwrap());
}
