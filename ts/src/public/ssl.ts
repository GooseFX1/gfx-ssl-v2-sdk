import { Program, Wallet } from "@project-serum/anchor";
import { Connection } from "@solana/web3.js";
import { ConstantIDs, getPoolRegistry, getSSLProgram } from "./utils";

export class SSL {
  connection: Connection;
  program: Program;
  wallet: Wallet;
  ADDRESSES: ConstantIDs

  constructor(connection: Connection, wallet: Wallet){
    this.connection = connection;
    this.wallet = wallet
    const program = getSSLProgram(connection, wallet)
    this.program = program
    this.initializeConstants()
  }

  initializeConstants(){
    this.ADDRESSES.POOL_REGISTRY = getPoolRegistry()
  }

}