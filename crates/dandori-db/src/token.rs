//! Băm token phiên: một nơi duy nhất để mọi crate băm và tra cứu phiên giống nhau.

use sha2::{Digest, Sha256};
use std::fmt::Write;

/// Băm token phiên bằng SHA-256 (hex 64 ký tự).
///
/// Token có entropy cao nên không cần băm chậm; DB chỉ giữ bản băm để lộ DB không đồng nghĩa với
/// chiếm được phiên.
#[must_use]
pub fn hash_session_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}
