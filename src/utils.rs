use sha3::{ Digest, Keccak256 };

pub(crate) fn keccak256_hash_string(from: String) -> String {
    format!("{:x}", Keccak256::digest(from.as_bytes()))
}