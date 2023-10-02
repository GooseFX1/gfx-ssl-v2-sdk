import {
    SystemProgram,
    TransactionInstruction,
} from "@solana/web3.js";

import {
    NATIVE_MINT,
    getAssociatedTokenAddress,
    createSyncNativeInstruction,
    createCloseAccountInstruction,
} from "@solana/spl-token";
import { closeAccount } from "@solana/spl-token";

import * as anchor from "@project-serum/anchor";
import { findAssociatedTokenAddress } from "./utils";

export const wrapSOLIx = (
    owner: anchor.web3.PublicKey,
    amount: number | bigint
) => {

    let wrappedSOLAta = findAssociatedTokenAddress(
        owner,
        NATIVE_MINT // mint
    );

    let ixs: TransactionInstruction[] = [];

    // Create a transfer instruction to the W_SOL ATA
    let transferIx = SystemProgram.transfer({
        fromPubkey: owner,
        toPubkey: wrappedSOLAta,
        lamports: amount
    });

    ixs.push(transferIx);

    // Sync the SOL balance with wrapped SOL balance on the ATA
    let syncSOLIx = createSyncNativeInstruction(wrappedSOLAta);
    
    ixs.push(syncSOLIx);

    return ixs;
}


// TODO - Might have to check with other references if this is the correct approach
// to unwrap SOL
// Source: https://solana.stackexchange.com/questions/1112/how-to-unwrap-wsol-to-sol/1118#1118
export const unwrapSOLIx = (
    owner: anchor.web3.PublicKey,
    amount: number | bigint
) => {
    // Unwrap all WSOL by closing the wrappedSOLAta

    const wrappedSOLAta = findAssociatedTokenAddress(
        owner,
        NATIVE_MINT
    );

    const closeWrappedSOLAtaIx = createCloseAccountInstruction(
        wrappedSOLAta,
        owner,
        owner
    );

    return closeWrappedSOLAtaIx;
}