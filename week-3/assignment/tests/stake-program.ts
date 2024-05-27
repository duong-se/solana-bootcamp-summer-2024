import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakeProgram } from "../target/types/stake_program";
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createMint,
  createMintToInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

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

    const rewardVaultAccount = await getAccount(provider.connection, rewardVault)
    console.log({ rewardVaultAccount })
  });
});
