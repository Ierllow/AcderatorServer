use super::AppError;
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

const TOKEN_BYTES: usize = 32;

#[derive(Debug, Eq, PartialEq)]
pub enum PasswordMatch {
    Current,
    Legacy,
    Invalid,
}

pub fn generate_token() -> String {
    let mut bytes = [0_u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    hex(&bytes)
}

pub fn hash_token(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::ServiceFailure(format!("password hashing failed: {error}")))
}

pub fn verify_password(stored: &str, candidate: &str) -> PasswordMatch {
    if stored.starts_with("$argon2") {
        return PasswordHash::new(stored)
            .ok()
            .filter(|hash| {
                Argon2::default()
                    .verify_password(candidate.as_bytes(), hash)
                    .is_ok()
            })
            .map(|_| PasswordMatch::Current)
            .unwrap_or(PasswordMatch::Invalid);
    }

    if constant_time_eq(stored.as_bytes(), candidate.as_bytes()) {
        PasswordMatch::Legacy
    } else {
        PasswordMatch::Invalid
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let length = left.len().max(right.len());
    for index in 0..length {
        let left_byte = left.get(index).copied().unwrap_or_default();
        let right_byte = right.get(index).copied().unwrap_or_default();
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}
