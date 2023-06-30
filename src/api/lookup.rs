use super::model::*;
use crate::{store::Store, utils::convert_addr_record_to_banano_address, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ens_gateway_server::db::Database;
use ethers::types::U256;
use std::collections::HashMap;
use tracing::info;

pub const DOMAIN: &str = "banano-testing.cc";
pub const BANANO_COIN_TYPE_SLIP44: u16 = 198;

/// Lookup Banano address of a BNS domain
#[utoipa::path(
    get,
    path = "/bns/lookup/{domain}",
    params(("domain", description = "BNS domain", example = "wtp")),
    responses(
        (status = 200, description = "Banano address", body = BNSLookupResponse,
            example = json!(BNSLookupResponse { banano_address: "ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj".to_string() })),
        (status = 404, description = "BNS domain not found", body = BNSError, example = json!(BNSError::NotFound(String::from("domain = wtp")))),
    ),
    tag = "lookup",
)]
#[tracing::instrument(name = "bns", skip(app_state))]
pub async fn lookup(
    State(app_state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Searching for BAN addy");
    if let Some(addr) = app_state
        .db
        .addr_coin_type(
            format!("{}.{}", domain, DOMAIN).as_str(),
            U256::from(BANANO_COIN_TYPE_SLIP44),
        )
        .await
    {
        let banano_address = convert_addr_record_to_banano_address(addr);
        info!("Resolved BAN addy: {}", banano_address);
        Ok((StatusCode::OK, Json(BNSLookupResponse { banano_address })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Lookup Banano addresses from multiple BNS domains
#[utoipa::path(
    post,
    path = "/bns/lookup",
    request_body(content = Vec<String>, description = "List of BNS domains", example = json!(["wtp", "yekta"])),
    responses(
        (status = 200, description = "Banano address", body = BNSBatchedResponse,
            example = json!({
                "wtp": "ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj",
                "yekta": "ban_1yekta1xn94qdnbmmj1tqg76zk3apcfd31pjmuy6d879e3mr469a4o4sdhd4"
            })),
    ),
    tag = "batch-lookup",
)]
#[tracing::instrument(name = "bns", skip(app_state))]
pub async fn batch_lookup(
    State(app_state): State<AppState>,
    Json(domains): Json<Vec<String>>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Searching for multiple BAN addresses");
    let mut mapping: HashMap<String, String> = HashMap::new();
    for domain in domains.iter() {
        if let Some(addr) = app_state
            .db
            .addr_coin_type(
                format!("{}.{}", domain, DOMAIN).as_str(),
                U256::from(BANANO_COIN_TYPE_SLIP44),
            )
            .await
        {
            let banano_address = convert_addr_record_to_banano_address(addr);
            info!("Resolved BAN addy: {}", banano_address);
            mapping.insert(domain.clone(), banano_address);
        } else {
            mapping.insert(domain.clone(), "".into());
        }
    }
    Ok((
        StatusCode::OK,
        Json(BNSBatchedResponse { addresses: mapping }),
    ))
}

/// Reverse lookup of a BNS domain from a Banano address
#[utoipa::path(
    get,
    path = "/bns/reverse-lookup/{banano_address}",
    params(("banano_address", description = "Banano address", example = "ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj")),
    responses(
        (status = 200, description = "BNS domain", body = BNSReverseLookupResponse, example = json!(BNSReverseLookupResponse { domain: "wtp".to_string() })),
        (status = 404, description = "BNS domain not found", body = BNSError,
            example = json!(BNSError::NotFound(String::from("banano = ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj")))),
    ),
    tag = "lookup",
)]
#[tracing::instrument(name = "bns", skip(app_state))]
pub async fn reverse_lookup(
    State(app_state): State<AppState>,
    Path(banano_address): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Searching for domain");
    if let Some(domain) = app_state
        .store
        .domain_with_banano_address(banano_address.as_str())
        .await
    {
        info!("Resolved domain: {}", domain);
        Ok((StatusCode::OK, Json(BNSReverseLookupResponse { domain })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Lookup BNS domains from multiple Banano addresses
#[utoipa::path(
    post,
    path = "/bns/reverse-lookup",
    request_body(content = Vec<String>, description = "List of Banano addresses", example = json!([
        "ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj",
        "ban_1yekta1xn94qdnbmmj1tqg76zk3apcfd31pjmuy6d879e3mr469a4o4sdhd4"
    ])),
    responses(
        (status = 200, description = "BNS domains", body = BNSBatchedResponse,
            example = json!({
                "ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj": "wtp",
                "ban_1yekta1xn94qdnbmmj1tqg76zk3apcfd31pjmuy6d879e3mr469a4o4sdhd4": "yekta"
            })),
        (status = 404, description = "BNS domain not found", body = BNSError, example = json!(BNSError::NotFound(String::from("domain = wtp")))),
    ),
    tag = "batch-lookup",
)]
#[tracing::instrument(name = "bns", skip(app_state))]
pub async fn batch_reserve_lookup(
    State(app_state): State<AppState>,
    Json(banano_addresses): Json<Vec<String>>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Searching for multiple BNS domains");
    let mut mapping: HashMap<String, String> = HashMap::new();
    for banano_address in banano_addresses.iter() {
        if let Some(domain) = app_state
            .store
            .domain_with_banano_address(banano_address.as_str())
            .await
        {
            mapping.insert(banano_address.clone(), domain.clone());
        } else {
            mapping.insert(banano_address.clone(), "".into());
        }
    }

    Ok((
        StatusCode::OK,
        Json(BNSBatchedResponse { addresses: mapping }),
    ))
}
