mod anchor_extensions;
mod decimals;
mod testing;

pub use anchor_extensions::{AnchorClientErrorExt, GetProgramAccounts};
pub use decimals::ApplyDecimal;
pub use testing::{
    admin_wallet, cluster, commitment_level, create_ata, create_token, create_wallet, mint_to,
    rpc_url, set_rpc_url, user_wallet,
};

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

#[throws(Error)]
pub fn load_keypair(src: &str) -> Keypair {
    let maybe_keypair = shellexpand::full(&src)
        .map_err(|e| anyhow!(e))
        .and_then(|path| -> Result<_, Error> { Ok(PathBuf::from(&*path).canonicalize()?) })
        .and_then(|path| read_keypair_file(&path).map_err(|_| anyhow!("Cannot read keypair")));

    match maybe_keypair {
        Ok(keypair) => keypair,
        Err(_) => Keypair::from_bytes(&bs58::decode(src).into_vec()?)?,
    }
}
