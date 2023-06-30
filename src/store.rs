use crate::api::{BANANO_COIN_TYPE_SLIP44, DOMAIN};
use crate::utils::get_public_key_from_banano_address;
use async_trait::async_trait;
use serde_json::Value;
use std::{fs::File, io::BufReader, path::PathBuf};

#[async_trait]
pub trait Store {
    /// Get BNS domain from Banano address
    async fn domain_with_banano_address(&self, banano_address: &str) -> Option<String>;
}

pub struct JsonStore {
    json: Value,
}

impl JsonStore {
    pub fn new(file: &PathBuf) -> Self {
        let file = File::open(file).expect("Can't open file");
        let reader = BufReader::new(file);
        let json = serde_json::from_reader(reader).expect("Can't parse Json file");
        JsonStore { json }
    }
}

#[async_trait]
impl Store for JsonStore {
    async fn domain_with_banano_address(&self, banano_address: &str) -> Option<String> {
        if let Some(public_key) = get_public_key_from_banano_address(banano_address) {
            let public_key = format!("0x{}", public_key);
            let domains = self.json.as_object().unwrap();
            let suffix = format!(".{}", DOMAIN);
            domains
                .iter()
                .filter_map(|(domain, data)| {
                    if let Some(addresses) = data.get("addresses") {
                        if let Some(ban_public_key) =
                            addresses.get(BANANO_COIN_TYPE_SLIP44.to_string())
                        {
                            if public_key.eq_ignore_ascii_case(ban_public_key.as_str().unwrap()) {
                                let domain =
                                    domain.strip_suffix(suffix.as_str()).unwrap().to_string();
                                Some(domain)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonStore, Store};
    use serde_json::json;

    #[tokio::test]
    async fn find_domain_with_banano_address() {
        let json = json!({
            "another-monkey.banano-testing.cc": {
                "addresses": {
                    "60": "0xc2B286Fb1141151928c86a9131B6BBfB7ab42CFf",
                },
            },
            "wtp.banano-testing.cc": {
                "addresses": {
                    "198": "0x53E21B083E50DBAEF8463C194C6CAAA6205C85D2D47C3F0198B976519212035D",
                },
            },
        });
        let store = JsonStore { json };

        let domain = store
            .domain_with_banano_address(
                "ban_1nz45e65wn8uouw6eh1sbjpcobj1dk4x7o5w9w1sjgdpc8b361txr4h1qtoj",
            )
            .await
            .unwrap();
        assert_eq!(domain, "wtp");
    }
}
