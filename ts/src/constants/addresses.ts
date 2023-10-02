import { PublicKey } from "@solana/web3.js";
import ssl_idl from "../idl/gfx_ssl_v2.json";

export const GFX_PROGRAM_ID = new PublicKey(ssl_idl.metadata.address)

export const AUTHORITY = new PublicKey("GeSkmvDED55EjnybgdN1gJ89p5V5H9W6jrrhxbZ1pDhQ")