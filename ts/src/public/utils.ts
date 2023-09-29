import { PublicKey, Connection } from "@solana/web3.js";
import ssl_idl from "../idl/gfx_ssl_v2.json";
import anchor from "@project-serum/anchor";
import { AUTHORITY, MINT_NAME_MAPPING } from "../constants";

export type ConstantIDs = {
  POOL_REGISTRY: PublicKey;
};

export type PairMints = {
  mint1: string,
  name1: string,
  mint2: string,
  name2: string
}

export type MintName = {
  mint: string,
  name: string
}

export const getSSLProgram = (connection: Connection, wallet: anchor.Wallet): anchor.Program => {
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "finalized",
  });
  const sslPorgarmId = ssl_idl.metadata.address;
  const program = new anchor.Program(ssl_idl as any, sslPorgarmId, provider);
  return program
}

export const getSSLProgramId = (): PublicKey => {
  return new PublicKey(ssl_idl.metadata.address)
};

export const getPoolRegistry = (): PublicKey => {
  try{
    const poolRegistryKey: [PublicKey, number] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("pool_registry"),
          AUTHORITY.toBuffer(),
        ],
        getSSLProgramId()
      );
    return poolRegistryKey[0];
  }
  catch(e){
    return null
  }
}

export const getPairAccountKeys = (
  poolRegistry: PublicKey,
  tokenMintOne: PublicKey,
  tokenMintTwo: PublicKey
): PublicKey => {
  try {
    const poolRegistryAccountKey = poolRegistry;
    const [key, _]: [PublicKey, number] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("pair"),
          poolRegistryAccountKey.toBuffer(),
          tokenMintOne.toBuffer(),
          tokenMintTwo.toBuffer(),
        ],
        getSSLProgramId()
      );
    return key;
  } catch (err) {
    return undefined;
  }
};

export const getNameFromMint = (mint: string) => {
  for (let i=0 ; i < MINT_NAME_MAPPING.length; i++){
    const item = MINT_NAME_MAPPING[i]
    if (item.mint === mint)
      return item.name
  }
  return null
}

export const getMintFromName = (name: string) => {
  for (let i=0 ; i < MINT_NAME_MAPPING.length; i++){
    const item = MINT_NAME_MAPPING[i]
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
      getSSLProgramId()
    )
    return liquidityAccountKey[0]
  } catch (err) {
    return undefined
  }
}