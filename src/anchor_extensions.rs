use anchor_client::{ClientError as AnchorClientError, Program};
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use anyhow::{anyhow, Error};
use bytemuck::{from_bytes, Pod};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, MemcmpEncoding, RpcFilterType},
    rpc_request::{RpcError, RpcResponseErrorData},
    rpc_response::RpcSimulateTransactionResult,
};
use solana_sdk::{instruction::InstructionError, transaction::TransactionError};
use std::convert::TryInto;
use std::mem::size_of;

pub trait AnchorClientErrorExt {
    fn code(&self) -> Option<u32>;
    fn canonicalize<E>(&self) -> Error
    where
        u32: TryInto<E>,
        E: std::fmt::Display;
}
impl AnchorClientErrorExt for AnchorClientError {
    fn code(&self) -> Option<u32> {
        match self {
            AnchorClientError::SolanaClientError(ClientError {
                kind:
                    ClientErrorKind::RpcError(RpcError::RpcResponseError {
                        data:
                            RpcResponseErrorData::SendTransactionPreflightFailure(
                                RpcSimulateTransactionResult {
                                    err:
                                        Some(TransactionError::InstructionError(
                                            _,
                                            InstructionError::Custom(code),
                                        )),
                                    ..
                                },
                            ),
                        ..
                    }),
                ..
            }) => Some(*code),
            _ => None,
        }
    }

    fn canonicalize<E>(&self) -> Error
    where
        u32: TryInto<E>,
        E: std::fmt::Display,
    {
        match self.code() {
            Some(c) => anyhow!(format_error_code::<E>(c)),
            None => anyhow!("{}", self),
        }
    }
}

pub fn format_error_code<E>(code: u32) -> String
where
    u32: TryInto<E>,
    E: std::fmt::Display,
{
    if (0..20).contains(&code) {
        let e: spl_token::error::TokenError = unsafe { std::mem::transmute(code as u8) };
        format!("SPL Error: {}", e)
    } else if (100..ERROR_CODE_OFFSET).contains(&code) {
        let e: ErrorCode = unsafe { std::mem::transmute(code) };
        format!("Anchor Error: {}", e)
    } else if let Ok(e) = TryInto::<E>::try_into(code) {
        format!("GFX Error: {}", e)
    } else {
        format!("Unknown code: {}", code)
    }
}

pub trait GetProgramAccounts {
    fn get_program_accounts<T: Default + Discriminator + AnchorSerialize + AccountDeserialize>(
        &self,
        filters: &[Memcmp],
    ) -> std::result::Result<Vec<(Pubkey, T)>, Error>;

    fn get_program_accounts_zero_copy<T: Default + Discriminator + Pod>(
        &self,
        filters: &[Memcmp],
    ) -> std::result::Result<Vec<(Pubkey, T)>, Error>;
}

impl GetProgramAccounts for Program {
    fn get_program_accounts<T: Default + Discriminator + AnchorSerialize + AccountDeserialize>(
        &self,
        filters: &[Memcmp],
    ) -> std::result::Result<Vec<(Pubkey, T)>, Error> {
        let rpc_client = self.rpc();

        let mut filters_ = vec![
            RpcFilterType::DataSize(8 + T::default().try_to_vec().unwrap().len() as u64),
            RpcFilterType::Memcmp(Memcmp {
                offset: 0,
                bytes: MemcmpEncodedBytes::Base58(bs58::encode(&T::discriminator()).into_string()),
                encoding: Some(MemcmpEncoding::Binary),
            }),
        ];

        for f in filters {
            filters_.push(RpcFilterType::Memcmp(f.clone()))
        }

        let accounts = rpc_client.get_program_accounts_with_config(
            &self.id(),
            RpcProgramAccountsConfig {
                filters: Some(filters_),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                with_context: None,
            },
        )?;

        let accounts = accounts
            .into_iter()
            .map(|(k, acc)| T::try_deserialize(&mut &*acc.data).map(|acc| (k, acc)))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(accounts)
    }

    fn get_program_accounts_zero_copy<T: Default + Discriminator + Pod>(
        &self,
        filters: &[Memcmp],
    ) -> std::result::Result<Vec<(Pubkey, T)>, Error> {
        let rpc_client = self.rpc();

        let mut filters_ = vec![
            RpcFilterType::DataSize(8 + size_of::<T>() as u64),
            RpcFilterType::Memcmp(Memcmp {
                offset: 0,
                bytes: MemcmpEncodedBytes::Base58(bs58::encode(&T::discriminator()).into_string()),
                encoding: Some(MemcmpEncoding::Binary),
            }),
        ];

        for f in filters {
            filters_.push(RpcFilterType::Memcmp(f.clone()))
        }

        let accounts = rpc_client.get_program_accounts_with_config(
            &self.id(),
            RpcProgramAccountsConfig {
                filters: Some(filters_),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                with_context: None,
            },
        )?;

        let accounts = accounts
            .into_iter()
            .map(|(k, acc)| (k, from_bytes::<T>(&acc.data[8..]).clone()))
            .collect::<Vec<_>>();
        Ok(accounts)
    }
}
