# gfx-solana-utils ![build](https://img.shields.io/github/workflow/status/GooseFX1/gfx-solana-utils/ci)

This repo contains some utility functions/traits for calling Solana smart contracts.

Currently, it contains the following mods:

* anchor_extensions: AnchorClientErrorExt for easily extracting the error code and converting it to Anchor error, GetProgramAccounts for the missing get_program_accounts RPC method on Anchor.
* decimals: DecimalApply for dealing with SPL token decimal.
* testing: A bunch of functions for integration testing the Solana contract. e.g. create a throw-away account, create a mint etc.


