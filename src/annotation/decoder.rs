use encoding_rs::GB18030;

pub fn decode_gb18030(raw: &[u8]) -> String {
    GB18030.decode(&raw).0.to_string()
}
