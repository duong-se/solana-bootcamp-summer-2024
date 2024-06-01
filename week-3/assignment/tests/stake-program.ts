import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { StakeProgram } from "../target/types/stake_program";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";

describe("stake-program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.StakeProgram as Program<StakeProgram>;
  const staker = anchor.web3.Keypair.generate();
  let stakerTokenAccount: anchor.web3.PublicKey;
  const usdcMintKp = anchor.web3.Keypair.generate();

  before(async () => {
    {
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          staker.publicKey,
          anchor.web3.LAMPORTS_PER_SOL
        )
      );
    }

    {
      const tx = new anchor.web3.Transaction();

      const lamports = await getMinimumBalanceForRentExemptMint(
        provider.connection
      );

      const createMintIx = anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: usdcMintKp.publicKey,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      });

      const initMintIx = createInitializeMint2Instruction(
        usdcMintKp.publicKey,
        6,
        provider.publicKey,
        provider.publicKey,
        TOKEN_PROGRAM_ID
      );

      stakerTokenAccount = getAssociatedTokenAddressSync(
        usdcMintKp.publicKey,
        staker.publicKey
      );

      const createStakerTokenAccountIx =
        createAssociatedTokenAccountInstruction(
          staker.publicKey,
          stakerTokenAccount,
          staker.publicKey,
          usdcMintKp.publicKey
        );

      const mintToStakerIx = createMintToInstruction(
        usdcMintKp.publicKey,
        stakerTokenAccount,
        provider.publicKey,
        1000 * 10 ** 6,
        []
      );

      tx.add(
        ...[
          createMintIx,
          initMintIx,
          createStakerTokenAccountIx,
          mintToStakerIx,
        ]
      );

      const ts = await provider.sendAndConfirm(tx, [usdcMintKp, staker]);
      console.log("Your transaction signature", tx);
    }
  });

  it("Is initialized!", async () => {
    const rewardVault = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("reward")],
      program.programId
    )[0];
    const tx = await program.methods
      .initialize()
      .accounts({
        admin: provider.publicKey,
        mint: usdcMintKp.publicKey,
        rewardVault: rewardVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const rewardVaultAccount = await getAccount(
      provider.connection,
      rewardVault
    );
    console.log({ rewardVaultAccount });
  });

  it("stake", async () => {
    const stakeInfo = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), staker.publicKey.toBytes()],
      program.programId
    )[0];

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      usdcMintKp.publicKey,
      stakeInfo,
      true
    );

    const stakeAmount = new BN(100 * 10 ** 6);

    const tx = await program.methods
      .stake(stakeAmount)
      .accounts({
        staker: staker.publicKey,
        mint: usdcMintKp.publicKey,
        stakerTokenAccount: stakerTokenAccount,
        stakeInfo: stakeInfo,
        vaultTokenAccount: vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([staker])
      .rpc();

    console.log("Your transaction signature", tx);

    const stakerAccount = await getAccount(
      provider.connection,
      stakerTokenAccount
    );

    const vaultAccount = await getAccount(
      provider.connection,
      vaultTokenAccount
    );

    const stakeInfoAccount = await program.account.stakeInfo.fetch(stakeInfo);

    console.log({ stakerAccount });

    console.log({ vaultAccount });

    console.log({ stakeInfoAccount });
  });


  it("unstake", async () => {
    const stakeInfo = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), staker.publicKey.toBytes()],
      program.programId
    )[0];

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      usdcMintKp.publicKey,
      stakeInfo,
      true
    );

    const rewardVault = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("reward")],
      program.programId
    )[0];

    const mintTx = new anchor.web3.Transaction()
    const mintToRewardVaultIx = createMintToInstruction(
      usdcMintKp.publicKey,
      rewardVault,
      provider.publicKey,
      2000 * 10 ** 6,
      [],
    )
    mintTx.add(mintToRewardVaultIx)
    await provider.sendAndConfirm(mintTx)

    const tx = await program.methods
      .unstake()
      .accounts({
        staker: staker.publicKey,
        mint: usdcMintKp.publicKey,
        stakerTokenAccount: stakerTokenAccount,
        stakeInfo: stakeInfo,
        vaultTokenAccount: vaultTokenAccount,
        rewardVault: rewardVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([staker])
      .rpc();

    console.log("Your transaction signature", tx);

    const stakerAccount = await getAccount(
      provider.connection,
      stakerTokenAccount
    );

    const vaultAccount = await getAccount(
      provider.connection,
      vaultTokenAccount
    );

    const stakeInfoAccount = await program.account.stakeInfo.fetch(stakeInfo);

    console.log({ stakerAccount: stakerAccount.amount });

    console.log({ vaultAccount: vaultAccount.amount });

    console.log({ stakeInfoAccount: stakeInfoAccount.amount });
  });
});
