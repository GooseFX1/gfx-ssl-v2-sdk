const { Wallet, BN } = require('@project-serum/anchor')
const { PublicKey, Connection, Keypair, Transaction, sendAndConfirmTransaction, ComputeBudgetProgram } = require('@solana/web3.js')
const {SSL} = require('gfx-ssl-sdk')
const connection = new Connection("https://api.mainnet-beta.solana.com")
const fs = require('fs');
const keypair = fs.readFileSync("", 'utf8') //Input the keypair path for actions
const kp = Keypair.fromSecretKey(new Uint8Array(JSON.parse(keypair)))
const wallet = new Wallet(payer = kp);

const signAndSendTransaction = async(connection, instructions, signers ) => {
  const tr = new Transaction()
  const ix1 = ComputeBudgetProgram.setComputeUnitLimit({units: 1_000_000})
  tr.add(ix1)
  instructions.map(ix => tr.add(ix))
  const res = await sendAndConfirmTransaction(connection, tr,signers, {skipPreflight: true})
  return res
}

xtest("Perform a swap", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const mintIn = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  const mintOut = new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")
  const ix = await ssl.swapIx({tokenMintIn: mintIn, tokenMintOut: mintOut, amountIn: new BN(100000)})
  const resp = await signAndSendTransaction(connection,ix, [kp] )
  console.log("resp: ", resp)
})

xtest("Deposit funds", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const mintIn = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  const ixs = await ssl.depositIx({tokenMint: mintIn, amountIn: new BN(1000000)})
  const resp = await signAndSendTransaction(connection,ixs, [kp] )
  console.log("resp: ", resp)
})

xtest("Withdraw funds", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const mintIn = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  const ixs = await ssl.withdrawIx({tokenMint: mintIn, amountIn: new BN(1000000)})
  const resp = await signAndSendTransaction(connection,ixs, [kp] )
  console.log("resp: ", resp)
})

xtest("Claim individual Rewards", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const mintIn = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  const ixs = await ssl.claimRewardsIx({tokenMint: mintIn})
  const resp = await signAndSendTransaction(connection,ixs, [kp] )
  console.log("resp: ", resp)
})

xtest("Claim multiple Rewards", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const mint1 = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  const mint2 = new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")

  const ix1 = await ssl.claimRewardsIx({tokenMint: mint1})
  const ix2 = await ssl.claimRewardsIx({tokenMint: mint2})
  const resp = await signAndSendTransaction(connection,[...ix1, ...ix2], [kp] )
  console.log("resp: ", resp)
})

xtest("Get liquidity details", async() => {
  const ssl = new SSL(connection, wallet.publicKey)
  const mint1 = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")

  const res = await ssl.getLiquidityData({
    tokenMint: mint1, 
    walletToQuery: new PublicKey("")}) //Inout your wallet here!
  console.log(res)
})