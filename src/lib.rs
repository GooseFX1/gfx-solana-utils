mod anchor_extensions;
mod decimals;
mod testing;

pub use anchor_extensions::{AnchorClientErrorExt, GetProgramAccounts};
pub use decimals::DecimalApply;
pub use testing::{admin_wallet, create_token, user_wallet};

use anchor_lang::prelude::*;
use anyhow::{anyhow, Error};
use fehler::{throw, throws};
use solana_sdk::{signature::Keypair, signer::keypair::read_keypair_file};
use std::cmp::Ordering;
use std::path::PathBuf;

pub trait Duplicate {
    fn clone(&self) -> Self;
}

impl Duplicate for Keypair {
    fn clone(&self) -> Self {
        Keypair::from_bytes(&self.to_bytes()).unwrap()
    }
}

#[throws(Error)]
pub fn sort_token_pair((token_a, token_b): (Pubkey, Pubkey)) -> (Pubkey, Pubkey) {
    match token_a.cmp(&token_b) {
        Ordering::Less => (token_a, token_b),
        Ordering::Equal => throw!(anyhow!("The pair of tokens are equal")),
        Ordering::Greater => (token_b, token_a),
    }
}

pub fn load_keypair(src: &str) -> Keypair {
    let path = PathBuf::from(src);
    let path = path.canonicalize().unwrap();

    match read_keypair_file(&path) {
        Ok(keypair) => keypair,
        Err(_) => Keypair::from_base58_string(src),
    }
}
