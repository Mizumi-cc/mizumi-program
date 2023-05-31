import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MizumiProgram } from "../target/types/mizumi_program";
import {
  Connection,
  Keypair, 
  PublicKey, 
  SystemProgram, 
  SYSVAR_CLOCK_PUBKEY, 
  SYSVAR_RENT_PUBKEY,
  Transaction, 
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createTransferInstruction, 
  getAssociatedTokenAddressSync
} from "@solana/spl-token";
import { expect } from "chai";

enum MizumiStable {
  USDC = 0,
  USDT = 1
}

enum MizumiFiat {
  GHS = 0,
  USD = 1,
}

enum TransactionKind {
  Onramp = 0,
  Offramp = 1,
}

describe("mizumi-program", () => {
  let provider = anchor.AnchorProvider.env()
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  
  const program = anchor.workspace.MizumiProgram as Program<MizumiProgram>;

  const connection = new Connection("https://api.devnet.solana.com", "confirmed");

  it("Is initialized!", async () => {
    const usdc_pk = new PublicKey("FWA2a9TgjhkTZB1YRofc9QemGn5LbbikbEoAHwbNZBDf")
    const usdt_pk = new PublicKey("5NJ3dRwgGsEzKo6fwzqizqurjythX7dRALmD3bdUfQK2")
    const [usdc_vault_pda, sb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("usdc-vault"),
        usdc_pk.toBuffer(),
      ],
      program.programId
    ) 
    const [usdt_vault_pda, vb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("usdt-vault"),
        usdt_pk.toBuffer()
      ],
      program.programId
    )
    const tx = await program.methods
      .initialize()
      .accounts({
        usdc: usdc_pk,
        usdcVault: usdc_vault_pda,
        usdt: usdt_pk,
        usdtVault: usdt_vault_pda,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY
      })
      .rpc();
    console.log("initialization transaction signature", tx);

    // get associated token accounts
    const usdc_associated_acc = getAssociatedTokenAddressSync(
      usdc_pk, provider.wallet.publicKey, true
    )
    const usdt_associated_acc = getAssociatedTokenAddressSync(
      usdt_pk, provider.wallet.publicKey, true
    )

    // helper fcn
    async function print_state() {
      let balance = await connection.getTokenAccountBalance(usdc_associated_acc)
      console.log("usdc_vault token amount", balance.value.amount)

      balance = await connection.getTokenAccountBalance(usdt_associated_acc)
      console.log("usdt_vault token amount", balance.value.amount)
    }

    // fund vaults
    const usdcTransferIx = createTransferInstruction(
      usdc_associated_acc,
      usdc_vault_pda,
      provider.wallet.publicKey,
      10000,
    )
    const usdtTransferIx = createTransferInstruction(
      usdt_associated_acc,
      usdt_vault_pda,
      provider.wallet.publicKey,
      10000,
    )
    const blockhash = (await connection.getLatestBlockhash())
    const fundTx = new Transaction(
      {
        feePayer: provider.wallet.publicKey,
        blockhash: blockhash.blockhash,
        lastValidBlockHeight: blockhash.lastValidBlockHeight,
      },
    )
    fundTx.instructions = [usdcTransferIx, usdtTransferIx]
    const signature = await provider.sendAndConfirm(fundTx)
    console.log("fund vaults transaction", signature)

    await print_state()
  });

  it("creates a new user account", async () => {
    // NOTE: devnet wallet for testing - CBzqazDdVraumuLRBraVMULDxsha3zkx5FZZW31Lzx6v
    const admin = Keypair.fromSecretKey(Uint8Array.from([253, 246, 244, 126,  21,  44, 202, 166, 181,  96,
      105, 195, 127, 161, 149, 102,  48, 175,  77,  97,
      145,  86, 101,  51, 210, 215, 207, 237, 160, 236,
      124, 177,  87,  95,   0, 236, 166, 253,  50, 102,
      207, 248, 123, 105,  43,   9, 251, 191, 182, 203,
      203, 114,  91, 142, 128, 110, 112,  57, 163, 106,
      23, 109, 235, 177])
    );
    const [user_acc_pda, db] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("user-account"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.methods
      .newUser()
      .accounts({
        admin: admin.publicKey,
        userAccount: user_acc_pda,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      })
      .signers([admin])
      .rpc()
    console.log("new user transaction signature", tx)
  })

  it("creates a new swap account", async() => {
    // NOTE: devnet wallet for testing - CBzqazDdVraumuLRBraVMULDxsha3zkx5FZZW31Lzx6v
    const admin = Keypair.fromSecretKey(Uint8Array.from([253, 246, 244, 126,  21,  44, 202, 166, 181,  96,
      105, 195, 127, 161, 149, 102,  48, 175,  77,  97,
      145,  86, 101,  51, 210, 215, 207, 237, 160, 236,
      124, 177,  87,  95,   0, 236, 166, 253,  50, 102,
      207, 248, 123, 105,  43,   9, 251, 191, 182, 203,
      203, 114,  91, 142, 128, 110, 112,  57, 163, 106,
      23, 109, 235, 177])
    );
    const [user_acc_pda, db] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("user-account"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    )

    const swaps_count = (await program.account.userAccount.fetch(user_acc_pda)).swapsCount
    const new_swaps_count = swaps_count.add(new anchor.BN(1))

    const [swap_acc_pda, sb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("swap-account"),
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(`${new_swaps_count.toNumber()}`),
      ],
      program.programId
    )

    const userAcc = await program.account.userAccount.fetch(user_acc_pda)
    console.log(userAcc, 'user')
    
    const tx = await program.methods
      .newSwap(`${new_swaps_count.toNumber()}`)
      .accounts({
        admin: admin.publicKey,
        userAccount: user_acc_pda,
        newSwapAccount: swap_acc_pda,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([
        admin,
      ])
      .rpc();

    console.log("new swap transaction signature", tx)
  })


  it("initiates a swap", async () => {
    // NOTE: devnet tokens for testing
    const usdc_pk = new PublicKey("FWA2a9TgjhkTZB1YRofc9QemGn5LbbikbEoAHwbNZBDf")
    const usdt_pk = new PublicKey("5NJ3dRwgGsEzKo6fwzqizqurjythX7dRALmD3bdUfQK2")

    // NOTE: devnet wallet for testing - CBzqazDdVraumuLRBraVMULDxsha3zkx5FZZW31Lzx6v
    const admin = Keypair.fromSecretKey(Uint8Array.from([253, 246, 244, 126,  21,  44, 202, 166, 181,  96,
      105, 195, 127, 161, 149, 102,  48, 175,  77,  97,
      145,  86, 101,  51, 210, 215, 207, 237, 160, 236,
      124, 177,  87,  95,   0, 236, 166, 253,  50, 102,
      207, 248, 123, 105,  43,   9, 251, 191, 182, 203,
      203, 114,  91, 142, 128, 110, 112,  57, 163, 106,
      23, 109, 235, 177])
    );
    const [user_acc_pda, db] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("user-account"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    const swaps_count = (await program.account.userAccount.fetch(user_acc_pda)).swapsCount
    const [swap_acc_pda, sb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("swap-account"),
        provider.wallet.publicKey.toBuffer(),
        Buffer.from( `${swaps_count.toNumber()}`),
      ],
      program.programId
    )
    const [usdc_vault_pda, tb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("usdc-vault"),
        usdc_pk.toBuffer(),
      ],
      program.programId
    ) 
    const [usdt_vault_pda, vb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("usdt-vault"),
        usdt_pk.toBuffer()
      ],
      program.programId
    )

    // get associated token accounts
    const usdc_associated_acc = getAssociatedTokenAddressSync(
      usdc_pk, provider.wallet.publicKey, true
    )
    const usdt_associated_acc = getAssociatedTokenAddressSync(
      usdt_pk, provider.wallet.publicKey, true
    )

    // helper fcn
    async function print_state() {
      const userAcc = await program.account.userAccount.fetch(user_acc_pda)
      const swapAcc = await program.account.swapAccount.fetch(swap_acc_pda)
      console.log(swapAcc, 'swap')
      console.log(userAcc, 'user')
    }

    const amount = new anchor.BN(100)

    await print_state()

    const tx = await program.methods
      .initiateSwap({usdc: {}}, amount, {usd: {}}, {onramp: {}}, `${swaps_count.toNumber()}`)
      .accounts({
        admin: admin.publicKey,
        authority: provider.wallet.publicKey,
        authorityUsdc: usdc_associated_acc,
        authorityUsdt: usdt_associated_acc,
        userAccount: user_acc_pda,
        swapAccount: swap_acc_pda,
        usdc: usdc_pk,
        usdcVault: usdc_vault_pda,
        usdt: usdt_pk,
        usdtVault: usdt_vault_pda,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .signers([admin])
      .rpc();
    console.log("initiate swap transaction", tx);

    await print_state()
  })

  it('completes a swap', async () => {
    // NOTE: devnet wallet for testing - CBzqazDdVraumuLRBraVMULDxsha3zkx5FZZW31Lzx6v
    const admin = Keypair.fromSecretKey(Uint8Array.from([253, 246, 244, 126,  21,  44, 202, 166, 181,  96,
      105, 195, 127, 161, 149, 102,  48, 175,  77,  97,
      145,  86, 101,  51, 210, 215, 207, 237, 160, 236,
      124, 177,  87,  95,   0, 236, 166, 253,  50, 102,
      207, 248, 123, 105,  43,   9, 251, 191, 182, 203,
      203, 114,  91, 142, 128, 110, 112,  57, 163, 106,
      23, 109, 235, 177])
    );
    const [user_acc_pda, db] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("user-account"),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    const swaps_count = (await program.account.userAccount.fetch(user_acc_pda)).swapsCount
    const [swap_acc_pda, sb] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("swap-account"),
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(`${swaps_count.toNumber()}`),
      ],
      program.programId
    )

    const swap_acc = await program.account.swapAccount.fetch(swap_acc_pda)
    console.log(swap_acc, 'swap')

    const tx = await program.methods
      .completeSwap(true, new anchor.BN(100), `${swaps_count.toNumber()}`)
      .accounts({
        admin: admin.publicKey,
        authority: provider.wallet.publicKey,
        swapAccount: swap_acc_pda,
        userAccount: user_acc_pda,
        clock: SYSVAR_CLOCK_PUBKEY
      })
      .signers([admin])
      .rpc();
    
    const swap_status = (await program.account.swapAccount.fetch(swap_acc_pda)).settled
    console.log('swap account settled?', swap_status);
    expect(swap_status).to.be.true;
  })
})
