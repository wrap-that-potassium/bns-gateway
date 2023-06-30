use crate::store::JsonStore;
use axum::{
    routing::{get, post},
    Router,
};
use clap::{arg, command, value_parser, ArgGroup};
use color_eyre::Report;
use ens_gateway_server::db::JsonDatabase;
use ens_gateway_server::gateway::Gateway;
use ethers::signers::{LocalWallet, Signer};
use eyre::Result;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod store;
mod utils;

#[derive(Clone)]
pub struct AppState {
    db: Arc<JsonDatabase>,
    store: Arc<JsonStore>,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "BNS API",
            description = "BNS (Banano Name Service) API",
            license(name = "MIT", url = "https://raw.githubusercontent.com/wrap-that-potassium/bns-gateway/main/LICENSE"),
        ),
        servers(
            (url = "https://bns.banano-testing.cc"),
            (url = "https://bns.banano.cc"),
            (url = "http://localhost:8080"),
        ),
        paths(
            api::lookup,
            api::reverse_lookup,
            api::batch_lookup,
            api::batch_reserve_lookup,
        ),
        components(
            schemas(
                api::BNSLookupResponse,
                api::BNSReverseLookupResponse,
                api::BNSBatchedResponse,
                api::BNSError
            )
        ),
        /*
        modifiers(&SecurityAddon),
        */
        tags(
            (name = "lookup", description = "Lookup operations"),
            (name = "batch-lookup", description = "Batch Lookup operations"),
        )
    )]
    struct ApiDoc;

    let matches = command!()
        .about("BNS (Banano Name Service) Gateway")
        .arg(
            arg!(-k --privatekey <VALUE> "private key of the wallet allowed to sign offchain ENS record results")
            .required(true)
            .env("PRIVATE_KEY")
            .hide_env_values(true)
        )
        .arg(arg!(-t --ttl <VALUE> "TTL for signatures")
            .value_parser(value_parser!(u64))
            .default_value("300")
            .env("TTL")
        )
        .arg(arg!(-i --ip <VALUE> "server IP to bind to -- change it to 0.0.0.0 for all interfaces")
            .value_parser(value_parser!(IpAddr))
            .default_value("127.0.0.1")
            .env("LISTEN_IP")
        )
        .arg(arg!(-p --port <VALUE> "server port to bind to")
            .value_parser(value_parser!(u16).range(1..))
            .default_value("8080")
            .env("LISTEN_PORT")
        )
        .arg(arg!(--json <FILE> "Json file to use as a database").value_parser(value_parser!(PathBuf)))
        //.arg(arg!(--postgres <CONNECTION_STRING> "PostgreSQL connection string"))
        .group(
            ArgGroup::new("database")
                .required(true)
                .args(["json"/*, "postgres"*/]),
        )
        .get_matches();

    setup()?;

    let private_key = matches
        .get_one::<String>("privatekey")
        .expect("Missing private key");
    let ttl = *matches.get_one::<u64>("ttl").expect("Missing TTL");
    let ip_address = *matches.get_one::<IpAddr>("ip").expect("Missing IP address");
    let port = *matches.get_one::<u16>("port").expect("Missing port");

    let signer = private_key.parse::<LocalWallet>()?;
    info!("Signing wallet: {}", signer.address());

    let file = matches.get_one::<PathBuf>("json").expect("Can't find file");
    info!("Using Json database from {:?}", file);
    let db = JsonDatabase::new(file);
    let db = Arc::new(db);

    let store = JsonStore::new(file);
    let store = Arc::new(store);

    let server = Gateway::new(signer, ttl, ip_address, port, db.clone()).await?;

    let bns_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/bns/lookup/:domain", get(api::lookup))
        .route(
            "/bns/reverse-lookup/:banano_address",
            get(api::reverse_lookup),
        )
        .route("/bns/lookup", post(api::batch_lookup))
        .route("/bns/reverse-lookup", post(api::batch_reserve_lookup))
        .with_state(AppState {
            db: db.clone(),
            store: store.clone(),
        });

    info!("Starting BNS gateway...");
    server.start_with_extra_router(bns_router).await?;

    Ok(())
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .compact();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    Ok(())
}
