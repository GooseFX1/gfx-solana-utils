use anchor_lang::prelude::*;
use anyhow::Error;
use fehler::throws;
use once_cell::sync::OnceCell;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcRequestAirdropConfig};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    program_pack::Pack,
    signature::{Keypair, Signer},
    signer::keypair::read_keypair_file,
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::{instruction::initialize_mint, state::Mint};
use std::{env, path::PathBuf};

static RPC_URL: &str = "https://api.devnet.solana.com";
static ADMIN_WALLET: OnceCell<Keypair> = OnceCell::new();
static USER_WALLET: OnceCell<Keypair> = OnceCell::new();

#[throws(Error)]
pub fn admin_wallet() -> &'static Keypair {
    ADMIN_WALLET.get_or_try_init(|| -> Result<_, Error> {
        if let Ok(wallet) = env::var("ADMIN_WALLET") {
            let path = PathBuf::from(wallet);
            let path = path.canonicalize().unwrap();

            let keypair = read_keypair_file(&path).expect("Cannot read keyfile");

            return Ok(keypair);
        }

        let key = Keypair::new();
        let rpc = RpcClient::new(RPC_URL.to_string());
        rpc.request_airdrop_with_config(
            &key.pubkey(),
            1000000000,
            RpcRequestAirdropConfig {
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Finalized,
                }),
                recent_blockhash: None,
            },
        )?;
        Ok(key)
    })?
}

#[throws(Error)]
pub fn user_wallet() -> &'static Keypair {
    USER_WALLET.get_or_try_init(|| -> Result<_, Error> {
        if let Ok(wallet) = env::var("USER_WALLET") {
            let path = PathBuf::from(wallet);
            let path = path.canonicalize().unwrap();

            let keypair = read_keypair_file(&path).expect("Cannot read keyfile");

            return Ok(keypair);
        }

        let key = Keypair::new();
        let rpc = RpcClient::new(RPC_URL.to_string());
        rpc.request_airdrop_with_config(
            &key.pubkey(),
            1000000000,
            RpcRequestAirdropConfig {
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Finalized,
                }),
                recent_blockhash: None,
            },
        )?;
        Ok(key)
    })?
}

#[throws(Error)]
pub fn create_token(authority: &Keypair) -> Pubkey {
    let rpc = RpcClient::new(RPC_URL.to_string());
    let (bhash, _) = rpc.get_recent_blockhash()?;

    let mint = Keypair::new();
    let ix0 = create_account(
        &authority.pubkey(),
        &mint.pubkey(),
        rpc.get_minimum_balance_for_rent_exemption(Mint::LEN)?,
        Mint::LEN as u64,
        &spl_token::ID,
    );

    let ix1 = initialize_mint(&spl_token::ID, &mint.pubkey(), &authority.pubkey(), None, 8)?;

    let tx = Transaction::new_signed_with_payer(
        &[ix0, ix1],
        Some(&authority.pubkey()),
        &[authority, &mint],
        bhash,
    );

    rpc.send_and_confirm_transaction(&tx)?;

    mint.pubkey()
}
