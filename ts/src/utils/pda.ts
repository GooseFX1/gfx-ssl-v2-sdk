import { web3 } from "@project-serum/anchor";
import {
  GFX_PROGRAM_ID,
  POOL_REGISTRY_SEED,
  PAIR_SEED,
  ORACLE_PRICE_HISTORY_SEED,
  LIQUIDITY_ACCOUNT_SEED,
} from "../constants";

export const getPoolRegistryAddress = (
  adminAddress: web3.PublicKey
): web3.PublicKey => {
  const [poolRegistryAddr] = web3.PublicKey.findProgramAddressSync(
    [POOL_REGISTRY_SEED, adminAddress.toBuffer()],
    GFX_PROGRAM_ID
  );

  return poolRegistryAddr;
};

export const getPairAddress = (
  poolRegistryAddr: web3.PublicKey,
  mintOneAddr: web3.PublicKey,
  mintTwoAddr: web3.PublicKey
): web3.PublicKey => {
  const [pairAddr] = web3.PublicKey.findProgramAddressSync(
    [
      PAIR_SEED,
      poolRegistryAddr.toBuffer(),
      mintOneAddr.toBuffer(),
      mintTwoAddr.toBuffer(),
    ],
    GFX_PROGRAM_ID
  );

  return pairAddr;
};

export const getOraclePriceHistoryAddress = (
  poolRegistryAddr: web3.PublicKey,
  oracleAccountAddr: web3.PublicKey
): web3.PublicKey => {
  const [oraclePriceHistoryAddr] = web3.PublicKey.findProgramAddressSync(
    [
      ORACLE_PRICE_HISTORY_SEED,
      poolRegistryAddr.toBuffer(),
      oracleAccountAddr.toBuffer(),
    ],
    GFX_PROGRAM_ID
  );

  return oraclePriceHistoryAddr;
};

export const getLiquidityAccountAddress = (
  poolRegistryAddr: web3.PublicKey,
  mintAddr: web3.PublicKey,
  ownerAddr: web3.PublicKey
): web3.PublicKey => {
  const [liquidityAccountAddr] = web3.PublicKey.findProgramAddressSync(
    [
      LIQUIDITY_ACCOUNT_SEED,
      poolRegistryAddr.toBuffer(),
      mintAddr.toBuffer(),
      ownerAddr.toBuffer(),
    ],
    GFX_PROGRAM_ID
  );

  return liquidityAccountAddr;
};
