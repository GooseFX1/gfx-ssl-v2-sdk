import { PublicKey } from "@solana/web3.js";
import { MintName, PairMints } from "../public/utils";

export const AUTHORITY: PublicKey = new PublicKey("GeSkmvDED55EjnybgdN1gJ89p5V5H9W6jrrhxbZ1pDhQ")

export const PAIR_MINTS: PairMints[] = [
  {
      "mint1": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
      "name1": "MSOL",
      "mint2": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
      "name2": "BONK"
  },
  {
      "mint1": "So11111111111111111111111111111111111111112",
      "name1": "WSOL",
      "mint2": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
      "name2": "BONK"
  },
  {
      "mint1": "So11111111111111111111111111111111111111112",
      "name1": "WSOL",
      "mint2": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
      "name2": "MSOL"
  },
  {
      "mint1": "So11111111111111111111111111111111111111112",
      "name1": "WSOL",
      "mint2": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "name2": "USDC"
  },
  {
      "mint1": "So11111111111111111111111111111111111111112",
      "name1": "WSOL",
      "mint2": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      "name2": "USDT"
  },
  {
      "mint1": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
      "name1": "BONK",
      "mint2": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "name2": "USDC"
      
  },
  {
      "mint1": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
      "name1": "MSOL",
      "mint2": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "name2": "USDC"
  },
  {
      "mint1": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "name1": "USDC",
      "mint2": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      "name2": "USDT"
  },
  {
      "mint1": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
      "name1": "BONK",
      "mint2": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      "name2": "USDT"
      
  },
  {
      "mint1": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
      "name1": "MSOL",
      "mint2": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
      "name2": "USDT"
  }
]

export const MINT_NAME_MAPPING: MintName[] = [
  {
    mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
    name: "MSOL"
  },
  {
    mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    name: "BONK"
  },
  {
    mint: "So11111111111111111111111111111111111111112",
    name: "WSOL"
  },
  {
    mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    name: "USDT"
  },
  {
    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    name: "USDC"
  }
]