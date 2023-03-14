import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SynthiusV0 } from "../target/types/synthius_v0";
import fs from "fs";
import assert from "assert";
import { PublicKey } from "@solana/web3.js";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { ClockworkProvider } from "@clockwork-xyz/sdk";

describe("synthius_v0", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;
  const clockworkProvider = new ClockworkProvider(payer, provider.connection);

  const program = anchor.workspace.SynthiusV0 as Program<SynthiusV0>;
  const programId = program.programId;
  let example_price = "G7dySNGaxZ8y2E89aX1K6rFeBt2ZnYBqXuCu1k2Y9MEe";

  const longMintKeypair = anchor.web3.Keypair.generate();
  const shortMintKeypair = anchor.web3.Keypair.generate();
  const collateralMintKeypair = anchor.web3.Keypair.generate();

  const depositedAmount = new anchor.BN(1);
  const dummyTokenAmount = new anchor.BN(5);

  const config = anchor.web3.Keypair.generate();
  const [vaultKey, vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode("vault")), payer.publicKey.toBuffer()], programId
  );
  const vaultWalletKey = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode("vaultWallet6")), payer.publicKey.toBuffer()], programId
  ) [0];

  const threadId = "counter-" + new Date().getTime() / 1000;
  const [threadAuthority] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("authority"), payer.publicKey.toBuffer()], 
    program.programId
  );
  const [threadAddress, threadBump] = clockworkProvider.getThreadPDA(threadAuthority, threadId)


  var programKey;
  try {
      let data = fs.readFileSync(
          '/Users/jannikspilker/Desktop/Grizzlython/synthius_v0/target/deploy/synthius_v0-keypair.json',
      );
      programKey = anchor.web3.Keypair.fromSecretKey(
          new Uint8Array(JSON.parse(data))
      );
  } catch (error) {
      throw new Error("Please make sure the program key is program_address.json.");
  }

  try {
      assert(programId.equals(programKey.publicKey));
  } catch (error) {
      throw new Error("Please make sure you have the same program address in Anchor.toml and program_address.json");
  }


  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize({
      loanPriceFeedId: new anchor.web3.PublicKey(example_price)
    }).accounts({
      program: programId,
      payer: payer.publicKey,
      config: config.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      vault: vaultKey, 
    }).signers([config, programKey]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Mints dummy token", async () => {
    const associatedTokenAddressCollateral = 
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});
    let tx = await program.methods.dummyToken(dummyTokenAmount).
            accounts({
              payer: payer.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              collateralTokenMint: collateralMintKeypair.publicKey,
              mintAuthority: payer.publicKey,
              collateralTokenAccount: associatedTokenAddressCollateral
            }).signers([payer.payer, collateralMintKeypair]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Buys Long", async () => {
    const associatedTokenAddressCollateral =
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});
    const associatedTokenAddressLongToken = 
          await anchor.utils.token.associatedAddress({mint: longMintKeypair.publicKey, owner: payer.publicKey});

    let tx = await program.methods.buyLong(depositedAmount)
            .accounts({
              config: config.publicKey,
              pythLoanAccount: new anchor.web3.PublicKey(example_price),
              payer: payer.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              longTokenMint: longMintKeypair.publicKey,
              mintAuthority: payer.publicKey,
              longTokenAccount: associatedTokenAddressLongToken,
              vault: vaultKey,
              collateralTokenMint: collateralMintKeypair.publicKey,
              collateralTokenAccount: associatedTokenAddressCollateral,
              vaultWallet: vaultWalletKey
            }).signers([payer.payer, longMintKeypair]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Sells Long", async() => {
    const associatedTokenAddressCollateral =
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});
    const associatedTokenAddressLongToken = 
          await anchor.utils.token.associatedAddress({mint: longMintKeypair.publicKey, owner: payer.publicKey});
          
    let tx = await program.methods.sellLong(vaultBump, payer.publicKey)
              .accounts({
                config: config.publicKey,
                pythLoanAccount: new anchor.web3.PublicKey(example_price),
                payer: payer.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
                tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                longTokenMint: longMintKeypair.publicKey,
                mintAuthority: payer.publicKey,
                longTokenAccount: associatedTokenAddressLongToken,
                collateralTokenMint: collateralMintKeypair.publicKey,
                collateralTokenAccount: associatedTokenAddressCollateral,
                vaultWallet: vaultWalletKey,
                vault: vaultKey,
                thread: threadAddress,
                threadAuthority: threadAuthority,
              }).signers([payer.payer]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Buys short",async () => {
    const associatedTokenAddressCollateral =
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});
    const associatedTokenAddressShortToken = 
          await anchor.utils.token.associatedAddress({mint: shortMintKeypair.publicKey, owner: payer.publicKey});
    
    let tx = await program.methods.buyShort(depositedAmount)
            .accounts({
              config: config.publicKey,
              pythLoanAccount: new anchor.web3.PublicKey(example_price),
              payer: payer.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              shortTokenMint: shortMintKeypair.publicKey,
              mintAuthority: payer.publicKey,
              shortTokenAccount: associatedTokenAddressShortToken,
              vault: vaultKey,
              collateralTokenMint: collateralMintKeypair.publicKey,
              collateralTokenAccount: associatedTokenAddressCollateral,
              vaultWallet: vaultWalletKey
            }).signers([payer.payer, shortMintKeypair]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Sells short",async () => {
    const associatedTokenAddressCollateral =
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});
    const associatedTokenAddressShortToken = 
          await anchor.utils.token.associatedAddress({mint: shortMintKeypair.publicKey, owner: payer.publicKey});
    let tx = await program.methods.sellShort(vaultBump, payer.publicKey)
            .accounts({
              config: config.publicKey,
              pythLoanAccount: new anchor.web3.PublicKey(example_price),
              payer: payer.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              vault: vaultKey,
              shortTokenMint: shortMintKeypair.publicKey,
              mintAuthority: payer.publicKey,
              shortTokenAccount: associatedTokenAddressShortToken,
              collateralTokenMint: collateralMintKeypair.publicKey,
              collateralTokenAccount: associatedTokenAddressCollateral,
              vaultWallet: vaultWalletKey,
              thread: threadAddress,
              threadAuthority: threadAuthority,
            }).signers([payer.payer]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Adds liquidity",async () => {
    const associatedTokenAddressCollateral =
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});

    let tx = await program.methods.addLiquidity(depositedAmount)
              .accounts({
                payer: payer.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
                tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                collateralTokenMint: collateralMintKeypair.publicKey,
                collateralTokenAccount: associatedTokenAddressCollateral,
                vault: vaultKey,
                vaultWallet: vaultWalletKey
              }).signers([payer.payer]).rpc();
    console.log("Your transaction signature", tx);
  })

  it ("Liquidates every 24 hours", async () => {
    const associatedTokenAddressCollateral =
          await anchor.utils.token.associatedAddress({mint: collateralMintKeypair.publicKey, owner: payer.publicKey});
    const associatedTokenAddressShortToken = 
          await anchor.utils.token.associatedAddress({mint: shortMintKeypair.publicKey, owner: payer.publicKey});
    const associatedTokenAddressLongToken = 
          await anchor.utils.token.associatedAddress({mint: longMintKeypair.publicKey, owner: payer.publicKey});
    let tx = await program.methods.trigger(Buffer.from(threadId),vaultBump, payer.publicKey)
            .accounts({
              config: config.publicKey,
              pythLoanAccount: new anchor.web3.PublicKey(example_price),
              payer: payer.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              vault: vaultKey,
              shortTokenMint: shortMintKeypair.publicKey,
              mintAuthority: payer.publicKey,
              shortTokenAccount: associatedTokenAddressShortToken,
              longTokenMint: longMintKeypair.publicKey,
              longTokenAccount: associatedTokenAddressLongToken,
              collateralTokenMint: collateralMintKeypair.publicKey,
              collateralTokenAccount: associatedTokenAddressCollateral,
              vaultWallet: vaultWalletKey,
              thread: threadAddress,
              threadAuthority: threadAuthority,
              clockworkProgram: clockworkProvider.threadProgram.programId,
            }).signers([payer.payer]).rpc();
    console.log("Your transaction signature", tx);
  });

});
