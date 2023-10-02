const { Wallet, BN } = require('@project-serum/anchor')
const { PublicKey, Connection, Keypair, Transaction, sendAndConfirmTransaction, ComputeBudgetProgram } = require('@solana/web3.js')
const {SSL} = require('gfx-ssl-sdk')
const connection = new Connection("https://api.mainnet-beta.solana.com")
const fs = require('fs');
const keypair = fs.readFileSync("/Users/arvindkrishnan/.config/solana/swap.json", 'utf8')
const kp = Keypair.fromSecretKey(new Uint8Array(JSON.parse(keypair)))
const wallet = new Wallet(payer = kp);

test("Testing example", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const sol = new PublicKey("So11111111111111111111111111111111111111112")
  const msol = new PublicKey("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So")
  const ix = await ssl.swapIx(sol, msol, new BN(100000))
  const tr = new Transaction()
  const ix1 = ComputeBudgetProgram.setComputeUnitLimit({units: 1_000_000})
  tr.add(ix1)
  tr.add(ix)
  const res = await sendAndConfirmTransaction(connection, tr,[kp], {skipPreflight: true})
  console.log("res: ", res)
})