import { BN, Program } from "@project-serum/anchor";
import { Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";
import { findAssociatedTokenAddress, getPoolRegistry, getSSLProgram, getValidPairKey, getSslPoolSignerKey, getOraclePriceHistory, getOracleFromMint, getFeeDestination, getLiquidityAccountKey, wrapSOLIx, unwrapAllSOLIx } from "./utils";
import { NATIVE_MINT, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "console";

export type SwapIxParams = {
  tokenMintIn: PublicKey;
  tokenMintOut: PublicKey;
  amountIn: BN;
  minAmountOut: BN;
};

export type CreateLiquidityAccountIxParams = {
  tokenMint: PublicKey;
}

export type DepositIxParams = {
  tokenMint: PublicKey;
  amountIn: BN;
  userAta?: PublicKey;
  useNativeSOL: boolean;
}

export type WithdrawIxParams = {
  tokenMint: PublicKey;
  amountIn: BN;
  userAta?: PublicKey;
  outNativeSOL?: boolean;
}

export class SSL {
  connection: Connection;
  program: Program;
  wallet: PublicKey;

  constructor(connection: Connection, wallet: PublicKey) {
    this.connection = connection;
    this.wallet = wallet;
    const program = getSSLProgram(connection);
    this.program = program;
  }

  async swapIx({
    tokenMintIn,
    tokenMintOut,
    amountIn,
    minAmountOut
  }: SwapIxParams): Promise<TransactionInstruction[]> {
    if (!this.connection) throw new Error("SSL Not initialized");
    const pair = getValidPairKey(
      tokenMintIn,
      tokenMintOut
    );
    if (!pair) throw new Error("Pair not supported");
    const userAtaIn = findAssociatedTokenAddress(this.wallet, tokenMintIn)
    const userAtaOut = findAssociatedTokenAddress(this.wallet, tokenMintOut)

    const sslPoolSignerIn = getSslPoolSignerKey(tokenMintIn)
    const sslPoolSignerOut = getSslPoolSignerKey(tokenMintOut)

    const inputOracle = getOracleFromMint(tokenMintIn)
    const outputOracle = getOracleFromMint(tokenMintOut)

    const priceHistoryIn = getOraclePriceHistory(inputOracle)
    const priceHistoryOut = getOraclePriceHistory(outputOracle)

    const sslOutMainVault = findAssociatedTokenAddress(sslPoolSignerOut, tokenMintOut)
    const sslOutSecondaryVault = findAssociatedTokenAddress(sslPoolSignerOut, tokenMintIn)

    const sslInMainVault = findAssociatedTokenAddress(sslPoolSignerIn, tokenMintIn)
    const sslInSecondaryVault = findAssociatedTokenAddress(sslPoolSignerIn, tokenMintOut)

    const feeVault = findAssociatedTokenAddress(getPoolRegistry(), tokenMintOut)
    const feeDestination = await getFeeDestination(pair, this.program, tokenMintOut)
    const accounts = {
      pair: pair,
      poolRegistry: getPoolRegistry(),
      userWallet: this.wallet,
      sslPoolInSigner: sslPoolSignerIn,
      sslPoolOutSigner: sslPoolSignerOut,
      userAtaIn: userAtaIn,
      userAtaOut: userAtaOut,
      sslOutMainVault: sslOutMainVault,
      sslOutSecondaryVault: sslOutSecondaryVault,
      sslInMainVault: sslInMainVault,
      sslInSecondaryVault: sslInSecondaryVault,
      sslOutFeeVault: feeVault,
      feeDestination: feeDestination,
      outputTokenPriceHistory: priceHistoryOut,
      outputTokenOracle: outputOracle,
      inputTokenPriceHistory: priceHistoryIn,
      inputTokenOracle: inputOracle,
      tokenProgram: TOKEN_PROGRAM_ID
    }
    const minOutVal = minAmountOut ? minAmountOut : new BN(0)
    return [(await this.program.methods.swap(amountIn, minOutVal).accounts(accounts).signers([]).instruction())]
  }

  async createLiquidityAccountIx({
    tokenMint
  }: CreateLiquidityAccountIxParams): Promise<TransactionInstruction[]> {
    const poolRegistry = getPoolRegistry();
    
    const liquidityAc = await getLiquidityAccountKey(this.wallet, tokenMint);

    const accounts = {
      poolRegistry: poolRegistry,
      mint: tokenMint,
      liquidityAccount: liquidityAc,
      owner: this.wallet
    }

    return [(await this.program.methods.createLiquidityAccountIx().accounts(accounts).instruction())];
  }

  async depositIx({
    tokenMint,
    amountIn,
    userAta,
    useNativeSOL
  }: DepositIxParams): Promise<TransactionInstruction[]> {

    let ixs: TransactionInstruction[] = [];

    const poolRegistry = getPoolRegistry();

    if(useNativeSOL) {
      assert(tokenMint.toString() === NATIVE_MINT.toString(), "The token mint must be W-SOL Pubkey if isNativeSOL = true");
      ixs.push(
        ...wrapSOLIx(this.wallet, amountIn.toNumber())
      );
    }

    const sslPoolSigner = getSslPoolSignerKey(tokenMint);

    const liquidityAc = await getLiquidityAccountKey(this.wallet, tokenMint);

    if((await this.connection.getBalance(liquidityAc)) === 0) {
      const createLiquidityAccountIx = (await this.createLiquidityAccountIx({tokenMint}));
      ixs.push(...createLiquidityAccountIx);
    }

    const poolVaultAc = findAssociatedTokenAddress(sslPoolSigner, tokenMint);

    const feeVault = findAssociatedTokenAddress(poolRegistry, tokenMint);

    const accounts = {
      liquidityAccount: liquidityAc,
      owner: this.wallet,
      userAta: userAta ? userAta : findAssociatedTokenAddress(this.wallet, tokenMint),
      sslPoolSigner: sslPoolSigner,
      poolVault: poolVaultAc,
      sslFeeVault: feeVault,
      poolRegistry: poolRegistry,
      tokenProgram: TOKEN_PROGRAM_ID
    };
  
    const depositIx = (await this.program.methods.depositIx(amountIn).accounts(accounts).signers([]).instruction());
    ixs.push(depositIx);

    return ixs;
  }

  async withdrawIx({
    tokenMint,
    amountIn,
    userAta,
    outNativeSOL
  }: WithdrawIxParams): Promise<TransactionInstruction[]> {
    let ixs: TransactionInstruction[] = [];

    const poolRegistry = getPoolRegistry();

    if(outNativeSOL) {
      assert(tokenMint.toString() === NATIVE_MINT.toString(), "The token mint must be W-SOL pubkey if outNativeSOL = true");
    }

    const sslPoolSigner = getSslPoolSignerKey(tokenMint);

    const liquidityAc = await getLiquidityAccountKey(this.wallet, tokenMint);

    const poolVaultAc = findAssociatedTokenAddress(sslPoolSigner, tokenMint);

    const feeVault = findAssociatedTokenAddress(poolRegistry, tokenMint);

    const accounts = {
      liquidityAccount: liquidityAc,
      owner: this.wallet,
      userAta: userAta ? userAta : findAssociatedTokenAddress(this.wallet, tokenMint),
      sslPoolSigner: sslPoolSigner,
      poolVault: poolVaultAc,
      sslFeeVault: feeVault,
      poolRegistry: poolRegistry,
      tokenProgram: TOKEN_PROGRAM_ID
    };

    let withdrawIx = await this.program.methods.withdrawIx(amountIn).accounts(accounts).signers([]).instruction();

    ixs.push(withdrawIx);

    if(outNativeSOL) {
      ixs.push(
        unwrapAllSOLIx(this.wallet)
      );
    }

    return ixs;
  }
}