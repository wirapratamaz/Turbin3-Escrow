import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram,
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";
import { 
  createMint, 
  getOrCreateAssociatedTokenAccount, 
  mintTo,
  getAccount,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import { assert } from "chai";

describe("escrow", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  
  // Generate keypairs for our test
  const maker = Keypair.generate();
  const taker = Keypair.generate();
  
  // Parameters for escrow
  const escrowSeed = new anchor.BN(Math.floor(Math.random() * 1000000));
  const depositAmount = new anchor.BN(50);
  const receiveAmount = new anchor.BN(100);
  
  // Store references to mints and token accounts
  let mintA: PublicKey;
  let mintB: PublicKey;
  let makerMintAAta: PublicKey;
  let makerMintBAta: PublicKey;
  let takerMintAAta: PublicKey;
  let takerMintBAta: PublicKey;
  
  // Store PDA references
  let escrowPDA: PublicKey;
  let vaultPDA: PublicKey;
  let escrowBump: number;
  
  before(async () => {
    // Airdrop SOL to maker and taker
    await provider.connection.requestAirdrop(maker.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(taker.publicKey, 10 * LAMPORTS_PER_SOL);
    
    // Wait for confirmation
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Create mint A
    mintA = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      0
    );
    
    // Create mint B
    mintB = await createMint(
      provider.connection,
      taker,
      taker.publicKey,
      null,
      0
    );
    
    // Create token accounts for maker
    const makerMintAAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      maker,
      mintA,
      maker.publicKey
    );
    makerMintAAta = makerMintAAccountInfo.address;
    
    const makerMintBAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      maker,
      mintB,
      maker.publicKey
    );
    makerMintBAta = makerMintBAccountInfo.address;
    
    // Create token accounts for taker
    const takerMintAAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      taker,
      mintA,
      taker.publicKey
    );
    takerMintAAta = takerMintAAccountInfo.address;
    
    const takerMintBAccountInfo = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      taker,
      mintB,
      taker.publicKey
    );
    takerMintBAta = takerMintBAccountInfo.address;
    
    // Mint tokens to maker and taker
    await mintTo(
      provider.connection,
      maker,
      mintA,
      makerMintAAta,
      maker.publicKey,
      depositAmount.toNumber()
    );
    
    await mintTo(
      provider.connection,
      taker,
      mintB,
      takerMintBAta,
      taker.publicKey,
      receiveAmount.toNumber()
    );
    
    // Derive escrow PDA
    [escrowPDA, escrowBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        escrowSeed.toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );

    // Note: The vault PDA will be created by the program using the ATA seeds
  });

  it("Initialize escrow", async () => {
    try {
      const tx = await program.methods
        .make(escrowSeed, receiveAmount, depositAmount)
        .accounts({
          maker: maker.publicKey,
          mintA: mintA,
          mintB: mintB,
          makerMintAAta: makerMintAAta,
          escrow: escrowPDA,
          vault: vaultPDA,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([maker])
        .rpc();
      
      console.log("Initialize escrow transaction signature", tx);
      
      // Verify escrow state
      const escrowAccount = await program.account.escrowState.fetch(escrowPDA);
      console.log("Escrow state:", escrowAccount);
      
      assert.equal(escrowAccount.maker.toString(), maker.publicKey.toString());
      assert.equal(escrowAccount.mintA.toString(), mintA.toString());
      assert.equal(escrowAccount.mintB.toString(), mintB.toString());
      assert.equal(escrowAccount.receiveAmount.toString(), receiveAmount.toString());
      assert.equal(escrowAccount.seed.toString(), escrowSeed.toString());
      
      // Find the vault ATA
      const vaultAddress = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        maker,
        mintA,
        escrowPDA,
        true
      ).then(account => account.address);
      
      // Check vault balance
      const vaultInfo = await getAccount(provider.connection, vaultAddress);
      assert.equal(vaultInfo.amount.toString(), depositAmount.toString());
      
    } catch (error) {
      console.error("Error initializing escrow:", error);
      throw error;
    }
  });

  it("Take escrow", async () => {
    try {
      const tx = await program.methods
        .take()
        .accounts({
          taker: taker.publicKey,
          mintA: mintA,
          mintB: mintB,
          maker: maker.publicKey,
          takerMintBAta: takerMintBAta,
          escrow: escrowPDA,
          vault: vaultPDA,
          takerMintAAta: takerMintAAta,
          makerMintBAta: makerMintBAta,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([taker])
        .rpc();
      
      console.log("Take escrow transaction signature", tx);
      
      // Verify token balances after the transaction
      const makerMintBInfo = await getAccount(provider.connection, makerMintBAta);
      const takerMintAInfo = await getAccount(provider.connection, takerMintAAta);
      
      assert.equal(makerMintBInfo.amount.toString(), receiveAmount.toString());
      assert.equal(takerMintAInfo.amount.toString(), depositAmount.toString());
      
    } catch (error) {
      console.error("Error taking escrow:", error);
      throw error;
    }
  });

  it("Refund escrow", async () => {
    // Create a new escrow for refund testing
    const refundSeed = new anchor.BN(Math.floor(Math.random() * 1000000));
    
    // Derive new escrow PDA
    const [refundEscrowPDA] = await PublicKey.findProgramAddress(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        refundSeed.toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );
    
    // Mint more tokens to maker for new escrow
    await mintTo(
      provider.connection,
      maker,
      mintA,
      makerMintAAta,
      maker.publicKey,
      depositAmount.toNumber()
    );
    
    // Initialize new escrow
    await program.methods
      .make(refundSeed, receiveAmount, depositAmount)
      .accounts({
        maker: maker.publicKey,
        mintA: mintA,
        mintB: mintB,
        makerMintAAta: makerMintAAta,
        escrow: refundEscrowPDA,
        vault: null, // This will be created by the program
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([maker])
      .rpc();
    
    // Find the vault for this escrow
    const refundVaultPDA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      maker,
      mintA,
      refundEscrowPDA,
      true
    ).then(account => account.address);
    
    // Now test refund
    try {
      const tx = await program.methods
        .refund()
        .accounts({
          mintA: mintA,
          maker: maker.publicKey,
          makerMintAAta: makerMintAAta,
          vault: refundVaultPDA,
          escrow: refundEscrowPDA,
          systemProgram: SystemProgram.programId,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([maker])
        .rpc();
      
      console.log("Refund escrow transaction signature", tx);
      
      // Verify maker got tokens back
      const makerMintAInfo = await getAccount(provider.connection, makerMintAAta);
      assert.equal(makerMintAInfo.amount.toString(), depositAmount.toString());
      
    } catch (error) {
      console.error("Error refunding escrow:", error);
      throw error;
    }
  });
}); 