use sha1::{Sha1, Digest};
use num_bigint::BigInt;

use std::iter;

pub fn server_hash(server_id: &str, secret: &[u8], public_key: &[u8]) -> String {
    let mut hasher = Sha1::new();

    hasher.update(server_id.as_bytes());
    hasher.update(secret);
    hasher.update(public_key);

    hexdigest(hasher)
}

pub fn hexdigest(hasher: Sha1) -> String {
    let output = hasher.finalize();

    let bigint = BigInt::from_signed_bytes_be(&output);
    format!("{:x}", bigint)
}