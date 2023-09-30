import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import ssl_idl from "../idl/gfx_ssl_v2.json";
import { Program, AnchorProvider, Wallet } from "@project-serum/anchor";
import { GFX_PROGRAM_ID, AUTHORITY, TOKEN_INFO, PAIR_MINTS, POOL_REGISTRY_SEED, PAIR_SEED, SSL_POOL, ORACLE_PRICE_HISTORY_SEED } from "../constants";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const getSSLProgram = (connection: Connection): Program => {
  const kp = Keypair.generate()
  const wallet = new Wallet(kp);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "finalized",
  });
  const sslPorgarmId = GFX_PROGRAM_ID;
  const program = new Program(ssl_idl as any, sslPorgarmId, provider);
  return program
}

export const getPoolRegistry = (): PublicKey => {
  try{
    const poolRegistryKey: [PublicKey, number] =
      PublicKey.findProgramAddressSync(
        [
          POOL_REGISTRY_SEED,
          AUTHORITY.toBuffer(),
        ],
        GFX_PROGRAM_ID
      );
    return poolRegistryKey[0];
  }
  catch(e){
    return null
  }
}

const getPairAccountKeys = (
  poolRegistry: PublicKey,
  tokenMintOne: PublicKey,
  tokenMintTwo: PublicKey
): PublicKey => {
  try {
    const poolRegistryAccountKey = poolRegistry;
    const [key, _]: [PublicKey, number] =
      PublicKey.findProgramAddressSync(
        [
          PAIR_SEED,
          poolRegistryAccountKey.toBuffer(),
          tokenMintOne.toBuffer(),
          tokenMintTwo.toBuffer(),
        ],
        GFX_PROGRAM_ID
      );
    return key;
  } catch (err) {
    return undefined;
  }
};

export const getNameFromMint = (mint: string) => {
  for (let i=0 ; i < TOKEN_INFO.length; i++){
    const item = TOKEN_INFO[i]
    if (item.mint === mint)
      return item.name
  }
  return null
}

export const getMintFromName = (name: string) => {
  for (let i=0 ; i < TOKEN_INFO.length; i++){
    const item = TOKEN_INFO[i]
    if (item.name === name)
      return item.mint
  }
  return null
}

export const getLiquidityAccountKey = async (
  walletKey: PublicKey,
  mint: PublicKey
): Promise<undefined | PublicKey> => {
  const poolRegistryAccountKey = getPoolRegistry()
  try {
    const liquidityAccountKey: [PublicKey, number] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("liquidity_account"),
        poolRegistryAccountKey.toBuffer(),
        mint.toBuffer(),
        walletKey.toBuffer()
      ],
      GFX_PROGRAM_ID
    )
    return liquidityAccountKey[0]
  } catch (err) {
    return undefined
  }
}

export const getValidPairKey = (
  tokenMintOne: PublicKey,
  tokenMintTwo: PublicKey
): PublicKey | null => {
  const poolRegistry = getPoolRegistry()
  for (let i=0; i< PAIR_MINTS.length - 1; i++){
      if (PAIR_MINTS[i].mint1 === tokenMintOne.toBase58() && PAIR_MINTS[i].mint2 === tokenMintTwo.toBase58()){
        return getPairAccountKeys(poolRegistry, tokenMintOne, tokenMintTwo )
      }
      else if (PAIR_MINTS[i].mint2 === tokenMintOne.toBase58() && PAIR_MINTS[i].mint1 === tokenMintTwo.toBase58()){
        return getPairAccountKeys(poolRegistry, tokenMintTwo, tokenMintOne )
      }
  }
  return undefined
};

export const findAssociatedTokenAddress = (
  walletAddress: PublicKey,
  tokenMintAddress: PublicKey
): PublicKey | null =>
  PublicKey.findProgramAddressSync(
    [
      walletAddress.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      tokenMintAddress.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )[0];

export const getSslPoolSignerKey = (
  tokenMintAddress: PublicKey
): undefined | PublicKey => {
  const poolRegistryAccountKey = getPoolRegistry();
  try {
    const sslPoolSignerKey: [PublicKey, number] =
      PublicKey.findProgramAddressSync(
        [
          SSL_POOL,
          poolRegistryAccountKey.toBuffer(),
          tokenMintAddress.toBuffer(),
        ],
        GFX_PROGRAM_ID
      );
    return sslPoolSignerKey[0];
  } catch (err) {
    return undefined;
  }
};

export const getOraclePriceHistory = (
  oracle: PublicKey
): undefined | PublicKey => {
  const poolRegistryAccountKey = getPoolRegistry();
  try {
    const priceHistoryKey: [PublicKey, number] =
      PublicKey.findProgramAddressSync(
        [
          ORACLE_PRICE_HISTORY_SEED,
          poolRegistryAccountKey.toBuffer(),
          oracle.toBuffer(),
        ],
        GFX_PROGRAM_ID
      );
    return priceHistoryKey[0];
  } catch (err) {
    return undefined;
  }
};

export const getOracleFromMint = (mint: PublicKey): PublicKey | undefined => {
  try {
    for (let i = 0; i < TOKEN_INFO.length; i++) {
      if (TOKEN_INFO[i].mint === mint.toBase58())
        return new PublicKey(TOKEN_INFO[i].oracle);
    }
    return undefined;
  } catch (e) {
    return undefined;
  }
};

export const getFeeDestination = async (
  pair: PublicKey,
  program: Program,
  outMint: PublicKey
) => {
  const pairInfo = await program.account.pair.fetch(pair);
  for (let i = 0; i < 2; i++) {
    if (pairInfo.mints[i].toBase58() === outMint.toBase58())
      return pairInfo.feeCollector[i];
  }
  return null
};