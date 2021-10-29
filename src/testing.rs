use crate::load_keypair;
use anchor_lang::prelude::*;
use anyhow::Error;
use fehler::throws;
use once_cell::sync::OnceCell;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcRequestAirdropConfig};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    system_program,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{initialize_mint, mint_to as spl_mint_to},
    state::Mint,
};
use std::env;

static RPC_URL: &str = "https://api.devnet.solana.com";
static ADMIN_WALLET: OnceCell<Keypair> = OnceCell::new();
static USER_WALLET: OnceCell<Keypair> = OnceCell::new();

#[throws(Error)]
pub fn admin_wallet() -> &'static Keypair {
    ADMIN_WALLET.get_or_try_init(|| -> Result<_, Error> {
        if let Ok(wallet) = env::var("ADMIN_WALLET") {
            return Ok(load_keypair(&wallet));
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
            return Ok(load_keypair(&wallet));
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

#[throws(Error)]
pub fn mint_to(mint: Pubkey, authority: &Keypair, to: Pubkey, amount: u64) {
    let rpc = RpcClient::new(RPC_URL.to_string());
    let (bhash, _) = rpc.get_recent_blockhash()?;

    let mut ixs = vec![];

    if rpc
        .get_account(&get_associated_token_address(&to, &mint))?
        .owner
        == system_program::ID
    {
        ixs.push(create_associated_token_account(
            &authority.pubkey(),
            &to,
            &mint,
        ));
    }

    ixs.push(spl_mint_to(
        &spl_token::ID,
        &mint,
        &to,
        &authority.pubkey(),
        &[&authority.pubkey()],
        amount,
    )?);

    let tx =
        Transaction::new_signed_with_payer(&ixs, Some(&authority.pubkey()), &[authority], bhash);

    rpc.send_and_confirm_transaction(&tx)?;
}
