use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Response from a forward lookup
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BNSLookupResponse {
    /// Banano Address
    pub banano_address: String,
}

/// Response from a reverse lookup
#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BNSReverseLookupResponse {
    /// BNS domain
    pub domain: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct BNSBatchedResponse {
    #[serde(flatten)]
    pub addresses: HashMap<String, String>,
}

/// BNS operation errors
#[derive(Serialize, Deserialize, ToSchema)]
pub enum BNSError {
    /// BNS domain not found
    #[schema(example = "domain = wtp")]
    NotFound(String),
}
