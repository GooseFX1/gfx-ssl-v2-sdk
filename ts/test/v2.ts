import * as anchor from "@project-serum/anchor";
import { SSL } from "../src";
import { describe, it } from "node:test";
import { ComputeBudgetProgram, sendAndConfirmTransaction } from "@solana/web3.js";
import { delay, getDecimalsFromMint } from "../src/public/utils";
import { assert } from "node:console";
import { TOKEN_INFO } from "../src/constants";

const fs = require('fs');
require('dotenv').config();

describe("Main-net tests on SSL-v2 SDK", () => {

    const connection = new anchor.web3.Connection("https://api.mainnet-beta.solana.com");

    const privateKey = fs.readFileSync("/Users/dhrumil/.config/solana/goosefx.json", "utf-8");
    const userKeypair = anchor.web3.Keypair.fromSecretKey(new Uint8Array(JSON.parse(privateKey)));
    const wallet =  new anchor.Wallet(userKeypair);
    console.log("User public key: ", wallet.publicKey);

    const WRAPPED_SOL = new anchor.web3.PublicKey('So11111111111111111111111111111111111111112');
    const USDC_MAINNET = new anchor.web3.PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');

    const sslClient = new SSL(connection, wallet.publicKey);

    it("Create a LiquidityAccount for a user", async() => {
        try {
            const txPacket = await sslClient.createLiquidityAccountIx({
                tokenMint: USDC_MAINNET
            });

            const tx = new anchor.web3.Transaction();
            
            for(let ix of txPacket.transactionInfos.ixs) {
                tx.add(ix);
            }

            const signature = await sendAndConfirmTransaction(connection, tx,[userKeypair, ...txPacket.transactionInfos.signers], {skipPreflight: true});
            console.log(`Create LiquidityAccount signature: https://solscan.io/tx/${signature}`);

            await delay(5_000);

            const liquidityAccount = await sslClient.program.account.liquidityAccount.fetch(txPacket.liquidityAccountAddr);

            //@ts-ignore
            assert(liquidityAccount.owner.toString() === userKeypair.publicKey.toString(), "Owner of the account does not match");
        }
        catch(err) {
            console.log("Error; ", err);
        }
    });

    it("Deposit some tokens to the vault", async() => {
        try {

            let usdcDecimals = getDecimalsFromMint(USDC_MAINNET.toString());
            let amountIn = new anchor.BN(1).mul(new anchor.BN(10 ** usdcDecimals));

            const txPacket = await sslClient.depositIx({
                tokenMint: USDC_MAINNET,
                amountIn,
            })

            const tx = new anchor.web3.Transaction();
            
            for(let ix of txPacket.transactionInfos.ixs) {
                tx.add(ix);
            }

            const signature = await sendAndConfirmTransaction(connection, tx,[userKeypair, ...txPacket.transactionInfos.signers], {skipPreflight: true});
            console.log(`Deposit tokens signature: https://solscan.io/tx/${signature}`);
        }
        catch(err) {
            console.log("Error; ", err);
        }
    });

    it("SHOULD FAIL: Premature closure a LiquidityAccount with funds", async() => {
        try {
            const txPacket = await sslClient.closeLiquidityAccountIx({
                tokenMint: USDC_MAINNET
            });

            const tx = new anchor.web3.Transaction();
            
            for(let ix of txPacket.transactionInfos.ixs) {
                tx.add(ix);
            }

            const signature = await sendAndConfirmTransaction(connection, tx,[userKeypair, ...txPacket.transactionInfos.signers], {skipPreflight: true});
            console.log(`Close LiquidityAccount signature: https://solscan.io/tx/${signature}`);
        }
        catch(err) {
            console.log("Error; ", err);
        }
    });

    it("Withdraw some tokens to the vault", async() => {
        try {

            let usdcDecimals = getDecimalsFromMint(USDC_MAINNET.toString());
            let amountIn = new anchor.BN(1).mul(new anchor.BN(10 ** usdcDecimals));

            const txPacket = await sslClient.withdrawIx({
                tokenMint: USDC_MAINNET,
                amountIn,
            })

            const tx = new anchor.web3.Transaction();
            
            for(let ix of txPacket.transactionInfos.ixs) {
                tx.add(ix);
            }

            const signature = await sendAndConfirmTransaction(connection, tx,[userKeypair, ...txPacket.transactionInfos.signers], {skipPreflight: true});
            console.log(`Withdraw tokens signature: https://solscan.io/tx/${signature}`);
        }
        catch(err) {
            console.log("Error; ", err);
        }
    });

    it('Swap some tokens', async() => {
        try {
            let usdcDecimals = getDecimalsFromMint(USDC_MAINNET.toString());
            let amountIn = new anchor.BN(1).mul(new anchor.BN(10 ** usdcDecimals));

            let tokenMintIn = USDC_MAINNET;
            let tokenMintOut = WRAPPED_SOL;

            const txPacket = await sslClient.swapIx({
                tokenMintIn,
                tokenMintOut,
                amountIn
            });

            const tx = new anchor.web3.Transaction();

            const extraUnitsIx = ComputeBudgetProgram.setComputeUnitLimit({units: 1_000_000});
            tx.add(extraUnitsIx);

            for(let ix of txPacket.transactionInfos.preIxs) {
                tx.add(ix);
            }

            for(let ix of txPacket.transactionInfos.ixs) {
                tx.add(ix);
            }

            const signature = await sendAndConfirmTransaction(connection, tx, [userKeypair, ...txPacket.transactionInfos.signers], {skipPreflight: true});
            console.log(`Swap USDC to WSOL signature: https://solscan.io/tx/${signature}`);
        }
        catch(err) {
            console.log("Error; ", err);
        }
    });

    it("Close a LiquidityAccount for a user", async() => {
        try {
            const txPacket = await sslClient.closeLiquidityAccountIx({
                tokenMint: USDC_MAINNET
            });

            const tx = new anchor.web3.Transaction();
            
            for(let ix of txPacket.transactionInfos.ixs) {
                tx.add(ix);
            }

            const signature = await sendAndConfirmTransaction(connection, tx,[userKeypair, ...txPacket.transactionInfos.signers], {skipPreflight: true});
            console.log(`Close LiquidityAccount signature: https://solscan.io/tx/${signature}`);
        }
        catch(err) {
            console.log("Error; ", err);
        }
    });
});