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
  useNativeSOL?: boolean;
}

export type WithdrawIxParams = {
  tokenMint: PublicKey;
  amountIn: BN;
  userAta?: PublicKey;
  outNativeSOL?: boolean;
}

export type ClaimFeesIxParams = {
  tokenMint: PublicKey;
  userAta?: PublicKey;
}

export type GetLiquidityParams = {
  tokenMint: PublicKey;
  walletToQuery?: PublicKey
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
    useNativeSOL = false
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
  
    const depositIx = (await this.program.methods.deposit(amountIn).accounts(accounts).signers([]).instruction());
    ixs.push(depositIx);

    return ixs;
  }

  async withdrawIx({
    tokenMint,
    amountIn,
    userAta,
    outNativeSOL = false
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

    let withdrawIx = await this.program.methods.withdraw(amountIn).accounts(accounts).signers([]).instruction();

    ixs.push(withdrawIx);

    if(outNativeSOL) {
      ixs.push(
        unwrapAllSOLIx(this.wallet)
      );
    }

    return ixs;
  }

  async claimRewardsIx({
    tokenMint,
    userAta,
  }: ClaimFeesIxParams): Promise<TransactionInstruction[]> {
    let ixs: TransactionInstruction[] = [];

    const poolRegistry = getPoolRegistry();
    const liquidityAc = await getLiquidityAccountKey(this.wallet, tokenMint);
    const feeVault = findAssociatedTokenAddress(poolRegistry, tokenMint);

    const accounts = {
      poolRegistry: poolRegistry,
      owner: this.wallet,
      sslFeeVault: feeVault,
      ownerAta: userAta ? userAta : findAssociatedTokenAddress(this.wallet, tokenMint),
      liquidityAccount: liquidityAc,
      tokenProgram: TOKEN_PROGRAM_ID
    };

    let claimFeesIx = await this.program.methods.claimFees().accounts(accounts).signers([]).instruction();
    ixs.push(claimFeesIx);
    return ixs;
  }

  async getLiquidityData({
    tokenMint,
    walletToQuery
  }: GetLiquidityParams) {

    const poolRegistry = getPoolRegistry()
    const sslPools = await this.program.account.poolRegistry.fetch(poolRegistry)
    let sslPool = null
    for (let i=0; i< (sslPools?.entries as any).length; i++){
      const token = sslPools.entries[i]
      if (token?.mint.toBase58() === tokenMint.toBase58()) {
        sslPool = token
      }
    }
    if (!sslPool) throw new Error("Pool not supported")

    const liquidityAc = await getLiquidityAccountKey(walletToQuery ?? this.wallet, tokenMint);
    const liquidityData = await this.program.account.liquidityAccount.fetch(liquidityAc)
    
    const amountDeposited = liquidityData.amountDeposited.toString()
    const totalEarned = liquidityData.totalEarned.toString()
    const lastClaimed = new Date((liquidityData.lastClaimed as BN).mul(new BN(1000)).toNumber())

    const diff = sslPool.totalAccumulatedLpReward.sub(liquidityData.lastObservedTap)
    const numerator = diff.mul(liquidityData.amountDeposited)
    const claimableAmount = numerator.div(sslPool.totalLiquidityDeposits).toString()
    return {
      amountDeposited, totalEarned, lastClaimed, claimableAmount, mint: tokenMint.toBase58()
    }
  }
}