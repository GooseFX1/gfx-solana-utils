use crate::decimals::ApplyDecimal;
use crate::load_keypair;
use anchor_lang::prelude::*;
use anyhow::Error;
use fehler::{throw, throws};
use num_traits::AsPrimitive;
use once_cell::sync::OnceCell;
use solana_client::{
    client_error::ClientErrorKind, rpc_client::RpcClient, rpc_config::RpcRequestAirdropConfig,
    rpc_request::RpcError,
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    program_pack::Pack,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{initialize_mint, mint_to as spl_mint_to},
    state::Mint,
};
use std::env;

static RPC_URL: OnceCell<String> = OnceCell::new();
static ADMIN_WALLET: OnceCell<Keypair> = OnceCell::new();
static USER_WALLET: OnceCell<Keypair> = OnceCell::new();

pub fn set_rpc_url(url: &str) -> bool {
    RPC_URL.set(url.to_string()).is_ok()
}

pub fn rpc_url() -> &'static str {
    RPC_URL
        .get_or_init(|| {
            env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())
        })
        .as_str()
}

#[throws(Error)]
pub fn admin_wallet(airdrop: f64) -> &'static Keypair {
    ADMIN_WALLET.get_or_try_init(|| -> Result<_, Error> {
        if let Ok(wallet) = env::var("ADMIN_WALLET") {
            return load_keypair(&wallet);
        }
        create_wallet(airdrop)
    })?
}

#[throws(Error)]
pub fn user_wallet(airdrop: f64) -> &'static Keypair {
    USER_WALLET.get_or_try_init(|| -> Result<_, Error> {
        if let Ok(wallet) = env::var("USER_WALLET") {
            return load_keypair(&wallet);
        }
        create_wallet(airdrop)
    })?
}

#[throws(Error)]
pub fn create_wallet(airdrop: f64) -> Keypair {
    let key = Keypair::new();
    if airdrop != 0f64 {
        let rpc = RpcClient::new(rpc_url().to_string());
        rpc.request_airdrop_with_config(
            &key.pubkey(),
            (airdrop * 1000000000.) as u64,
            RpcRequestAirdropConfig {
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Finalized,
                }),
                recent_blockhash: None,
            },
        )?;
    }
    key
}

#[throws(Error)]
pub fn create_token(authority: &Keypair) -> Pubkey {
    let rpc = RpcClient::new(rpc_url().to_string());
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
pub fn mint_to<N: AsPrimitive<f64>>(mint: Pubkey, authority: &Keypair, to: Pubkey, amount: N) {
    let rpc = RpcClient::new(rpc_url().to_string());
    let (bhash, _) = rpc.get_recent_blockhash()?;

    let mut ixs = vec![];

    if let Err(e) = rpc.get_account(&get_associated_token_address(&to, &mint)) {
        match e.kind() {
            ClientErrorKind::RpcError(RpcError::ForUser(_)) => {
                ixs.push(create_associated_token_account(
                    &authority.pubkey(),
                    &to,
                    &mint,
                ));
            }
            _ => throw!(e),
        }
    }
    let mint_account = rpc.get_account(&mint)?;
    let mint_account = Mint::unpack(&mint_account.data)?;

    ixs.push(spl_mint_to(
        &spl_token::ID,
        &mint,
        &get_associated_token_address(&to, &mint),
        &authority.pubkey(),
        &[&authority.pubkey()],
        mint_account.decimals.apply(amount),
    )?);

    let tx =
        Transaction::new_signed_with_payer(&ixs, Some(&authority.pubkey()), &[authority], bhash);

    rpc.send_and_confirm_transaction(&tx)?;
}

#[throws(Error)]
pub fn create_ata(mint: Pubkey, authority: &Keypair, to: Pubkey) {
    let rpc = RpcClient::new(rpc_url().to_string());
    let (bhash, _) = rpc.get_recent_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[create_associated_token_account(
            &authority.pubkey(),
            &to,
            &mint,
        )],
        Some(&authority.pubkey()),
        &[authority],
        bhash,
    );

    rpc.send_and_confirm_transaction(&tx)?;
}
