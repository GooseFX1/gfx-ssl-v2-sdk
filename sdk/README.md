

## GFX SSL v2 SDK

This crate provides the following features:
- `solana_sdk::Instruction` factory functions for SSLv2 program instructions
- Functions to fetch blockchain state for various types of SSLv2 program accounts.

It leverages types exposed in the `gfx-ssl-v2-interface` program crate,
which is a skeleton of the actual `gfx-ssl-v2` program containing the same
instructions and data types without the program logic.
