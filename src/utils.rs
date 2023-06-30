use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use data_encoding::Encoding;
use data_encoding_macro::new_encoding;
use ens_gateway_server::db::AddressBytesRecord;
use ethers::utils::hex;

const BANANO_BASE32: Encoding = new_encoding! {
    symbols: "13456789abcdefghijkmnopqrstuwxyz",
    check_trailing_bits: false,
};

pub fn convert_addr_record_to_banano_address(record: AddressBytesRecord) -> String {
    get_banano_address(record.addr.as_ref(), Some("ban_"))
}

pub fn get_banano_address(pub_key_bytes: &[u8], prefix: Option<&str>) -> String {
    let mut pub_key_vec = pub_key_bytes.to_vec();
    let mut h = [0u8; 3].to_vec();
    h.append(&mut pub_key_vec);
    let checksum = BANANO_BASE32.encode(&compute_address_checksum(pub_key_bytes));
    let address = {
        let encoded_addr = BANANO_BASE32.encode(&h);

        let mut addr = String::from("");
        if let Some(prefix) = prefix {
            addr = String::from(prefix);
        };
        addr.push_str(encoded_addr.get(4..).unwrap());
        addr.push_str(&checksum);
        addr
    };
    address
}

pub fn get_public_key_from_banano_address(banano_address: &str) -> Option<String> {
    let parts: Vec<&str> = banano_address.split('_').collect();
    if let Some(mut encoded_addr) = parts[1].get(0..52).map(String::from) {
        encoded_addr.insert_str(0, "1111");
        BANANO_BASE32
            .decode(encoded_addr.as_bytes())
            .ok()
            .map(hex::encode)
            .map(|public_key| public_key.split_at(6).1.to_string())
    } else {
        None
    }
}

fn compute_address_checksum(pub_key_bytes: &[u8]) -> [u8; 5] {
    let mut hasher = Blake2bVar::new(5).unwrap();
    let mut buf = [0u8; 5];
    hasher.update(pub_key_bytes);
    hasher.finalize_variable(&mut buf).unwrap();
    buf.reverse();
    buf
}

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use ethers::utils::hex;

    #[test]
    fn test_decoding() {
        assert_eq!(
            get_banano_address(
                hex::decode("0d7471e5d11faddce5315c97b23b464184afa8c4c396dcf219696b2682d0adf6")
                    .unwrap()
                    .as_ref(),
                None
            ),
            "15dng9kx49xfumkm4q6qpaxneie6oynebiwpums3ktdd6t3f3dhp69nxgb38"
        );
        assert_eq!(
            get_banano_address(
                hex::decode("2298fab7c61058e77ea554cb93edeeda0692cbfcc540ab213b2836b29029e23a")
                    .unwrap()
                    .as_ref(),
                None
            ),
            "1anrzcuwe64rwxzcco8dkhpyxpi8kd7zsjc1oeimpc3ppca4mrjtwnqposrs"
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            get_public_key_from_banano_address(
                "ban_15dng9kx49xfumkm4q6qpaxneie6oynebiwpums3ktdd6t3f3dhp69nxgb38"
            ),
            Some("0d7471e5d11faddce5315c97b23b464184afa8c4c396dcf219696b2682d0adf6".to_string()),
        );
        assert_eq!(
            get_public_key_from_banano_address(
                "ban_1anrzcuwe64rwxzcco8dkhpyxpi8kd7zsjc1oeimpc3ppca4mrjtwnqposrs"
            ),
            Some("2298fab7c61058e77ea554cb93edeeda0692cbfcc540ab213b2836b29029e23a".to_string()),
        );
    }

    #[test]
    fn test_convert_addr_record() {
        assert_eq!(
            convert_addr_record_to_banano_address(AddressBytesRecord {
                addr: hex::decode(
                    "0d7471e5d11faddce5315c97b23b464184afa8c4c396dcf219696b2682d0adf6"
                )
                .unwrap()
                .into(),
            }),
            "ban_15dng9kx49xfumkm4q6qpaxneie6oynebiwpums3ktdd6t3f3dhp69nxgb38"
        );
        assert_eq!(
            convert_addr_record_to_banano_address(AddressBytesRecord {
                addr: hex::decode(
                    "2298fab7c61058e77ea554cb93edeeda0692cbfcc540ab213b2836b29029e23a"
                )
                .unwrap()
                .into(),
            }),
            "ban_1anrzcuwe64rwxzcco8dkhpyxpi8kd7zsjc1oeimpc3ppca4mrjtwnqposrs"
        );
    }
}
