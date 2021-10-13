const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { SystemProgram, PublicKey } = anchor.web3;
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, Token } = require("@solana/spl-token");
const FAILED_TO_FIND_ACCOUNT = 'Failed to find account';
const INVALID_ACCOUNT_OWNER = 'Invalid account owner';

describe("ddca", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  // const provider = anchor.Provider.local(url="http://localhost:8899", opts= {
  //   preflightCommitment: "confirmed",
  //   commitment: "confirmed",
  // });
  anchor.setProvider(provider);

  //
  const DDCA_OPERATING_ACCOUNT_ADDRESS = new PublicKey("3oSfkjQZKCneYvsCTZc9HViGAPqR8pYr4h9YeGB5ZxHf");
  const SYSTEM_PROGRAM_ID = anchor.web3.SystemProgram.programId;
  const LAMPORTS_PER_SOL = anchor.web3.LAMPORTS_PER_SOL;
  const program = anchor.workspace.Ddca;
  const payer = anchor.web3.Keypair.generate();
  const ownerAccount = provider.wallet.payer; //anchor.web3.Keypair.generate(); 
  const ownerAccountAddress = ownerAccount.publicKey;
  const mintAuthority = anchor.web3.Keypair.generate();
  let mintA = null;
  let mintB = null;
  let splTokenAClient = null;
  let splTokenBClient = null;

  let ownerTokenAccountAAddress = null;
  const ownerTokenAccountAInitialBalance = 500;

  const ddcaFromInitialAmount = 100;
  const ddcaFromAmountPerSwap = 10;
  const ddcaIntervalInSeconds = 5 * 60; //5 minutes

  // Hybrid Liquidity Aggregator accounts
  const HLA_PROGRAM_ADDRESS = new PublicKey("B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3");
  const HLA_OPERATING_ACCOUNT_ADDRESS = new PublicKey("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
  const hlaProtocolAddress = anchor.web3.Keypair.generate().publicKey;
  const hlaPoolAddress = anchor.web3.Keypair.generate().publicKey;
  const hlaAmmAddress = anchor.web3.Keypair.generate().publicKey;

  it("One-time test setup", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );
    
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(ownerAccountAddress, 1000000000),
      "confirmed"
    );

    mintA = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      1,
      TOKEN_PROGRAM_ID
    );

    ownerTokenAccountAAddress = await mintA.createAssociatedTokenAccount(ownerAccountAddress);
    await mintA.mintTo(
      ownerTokenAccountAAddress,
      mintAuthority.publicKey,
      [mintAuthority],
      ownerTokenAccountAInitialBalance
    );
    let _ownerTokenAccountA = await mintA.getAccountInfo(ownerTokenAccountAAddress);
    assert.ok(_ownerTokenAccountA.amount.toNumber() == ownerTokenAccountAInitialBalance);

    mintB = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      1,
      TOKEN_PROGRAM_ID
    );

    splTokenAClient = new Token(
      program.provider.connection,
      mintA.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );

    splTokenBClient = new Token(
      program.provider.connection,
      mintB.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );

  });

  it("Should create ddca", async () => {
    //CREATE DDCA

    // const timestampSeed = Date.now().toString();
    const blockHeight = await program.provider.connection.getSlot('confirmed');
    const blockHeightBytes = new anchor.BN(blockHeight).toBuffer('be', 8);

    //ddca account pda and bump
    const [ddcaAccountPda, ddcaAccountPdaBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        ownerAccountAddress.toBuffer(),
        blockHeightBytes,
        Buffer.from(anchor.utils.bytes.utf8.encode("ddca-seed")),
      ],
      program.programId
    );

    //ddca associated token account (from)
    const ddcaFromTokenAccountAddress = await Token.getAssociatedTokenAddress(
      associatedProgramId=ASSOCIATED_TOKEN_PROGRAM_ID,
      programId=TOKEN_PROGRAM_ID,
      mint=mintA.publicKey,
      owner=ddcaAccountPda,
      allowOwnerOffCurve=true,
    );
    //ddca associated token account (to)
    const ddcaToTokenAccountAddress = await Token.getAssociatedTokenAddress(
      associatedProgramId=ASSOCIATED_TOKEN_PROGRAM_ID,
      programId=TOKEN_PROGRAM_ID,
      mint=mintB.publicKey,
      owner=ddcaAccountPda,
      allowOwnerOffCurve=true,
    );

    //ddca operating token account (from)
    const ddcaOperatingFromTokenAccountAddress = await Token.getAssociatedTokenAddress(
      associatedProgramId=ASSOCIATED_TOKEN_PROGRAM_ID,
      programId=TOKEN_PROGRAM_ID,
      mint=mintA.publicKey,
      owner=DDCA_OPERATING_ACCOUNT_ADDRESS,
    );

    //hla operating token account (from)
    const hlaOperatingFromTokenAccountAddress = await Token.getAssociatedTokenAddress(
      associatedProgramId=ASSOCIATED_TOKEN_PROGRAM_ID,
      programId=TOKEN_PROGRAM_ID,
      mint=mintA.publicKey,
      owner=HLA_OPERATING_ACCOUNT_ADDRESS,
    );

    // Instructions
    let instructions = [];

    let ddcaOperatingAtaCreateInstruction = await createAtaCreateInstructionIfNotExists(
      ddcaOperatingFromTokenAccountAddress, 
      mintA.publicKey, 
      DDCA_OPERATING_ACCOUNT_ADDRESS, 
      provider.wallet.payer.publicKey,
      splTokenAClient);
    if(ddcaOperatingFromTokenAccountAddress !== null)
      instructions.push(ddcaOperatingAtaCreateInstruction);
    
    let hlaOperatingAtaCreateInstruction = await createAtaCreateInstructionIfNotExists(
      hlaOperatingFromTokenAccountAddress, 
      mintA.publicKey, 
      HLA_OPERATING_ACCOUNT_ADDRESS, 
      provider.wallet.payer.publicKey,
      splTokenAClient);
    if(hlaOperatingAtaCreateInstruction !== null)
      instructions.push(hlaOperatingAtaCreateInstruction);

    if(instructions.length == 0)
      instructions = undefined;
    
    let ownerLamports = await getLamports(ownerAccountAddress);
    let payerLamports = await getLamports(payer.publicKey);

    console.log("TEST PARAMETERS:")
    console.log("  Program ID:                         " + program.programId);
    console.log("  payer.address:                      " + payer.publicKey);
    console.log("  payer.lamports:                     %s (%s SOL)", payerLamports, payerLamports / LAMPORTS_PER_SOL);
    console.log("  mintAuthority.publicKey:            " + mintAuthority.publicKey);
    console.log("  mintA.publicKey:                    " + mintA.publicKey);
    console.log("  mintB.publicKey:                    " + mintB.publicKey);
    console.log("  blockHeight:                        " + blockHeight);
    console.log();
    console.log("  ownerAccountAddress:                " + ownerAccountAddress);
    console.log("  ownerAccount.lamports:              %s (%s SOL)", ownerLamports, ownerLamports / LAMPORTS_PER_SOL);
    console.log("  ownerFromTokenAccountAddress:       " + ownerTokenAccountAAddress);
    console.log();
    console.log("  ddcaAccountPda:                     " + ddcaAccountPda);
    console.log("  ddcaAccountPdaBump:                 " + ddcaAccountPdaBump);
    console.log("  ddcaFromTokenAccountAddress:        " + ddcaFromTokenAccountAddress);
    console.log("  ddcaToTokenAccountAddress:          " + ddcaToTokenAccountAddress);
    console.log();
    console.log("  DDCA_OPERATING_ACCOUNT_ADDRESS:     " + DDCA_OPERATING_ACCOUNT_ADDRESS);
    console.log("  ddcaOperatingFromTokenAccountAddress: " + ddcaOperatingFromTokenAccountAddress);
    console.log();
    console.log("  HLA_PROGRAM_ADDRESS:               " + HLA_PROGRAM_ADDRESS);
    console.log("  hlaProtocolAddress:                " + hlaProtocolAddress);
    console.log("  hlaPoolAddress:                    " + hlaPoolAddress);
    console.log("  hlaAmmAddress:                     " + hlaAmmAddress);
    console.log("  HLA_OPERATING_ACCOUNT_ADDRESS:     " + HLA_OPERATING_ACCOUNT_ADDRESS);
    console.log("  hlaOperatingFromTokenAccountAddress: " + hlaOperatingFromTokenAccountAddress);
    console.log();
    console.log("  SYSTEM_PROGRAM_ID:                  " + SYSTEM_PROGRAM_ID);
    console.log("  TOKEN_PROGRAM_ID:                   " + TOKEN_PROGRAM_ID);
    console.log("  ASSOCIATED_TOKEN_PROGRAM_ID:        " + ASSOCIATED_TOKEN_PROGRAM_ID);
    console.log();

    const createTx = await program.rpc.create(new anchor.BN(blockHeight), ddcaAccountPdaBump,
      new anchor.BN(ddcaFromInitialAmount), new anchor.BN(ddcaFromAmountPerSwap), new anchor.BN(ddcaIntervalInSeconds), 
      new anchor.BN(0), 0,
      {
        accounts: {
          // owner
          ownerAccount: ownerAccountAddress,
          ownerFromTokenAccount: ownerTokenAccountAAddress,
          // ddca
          ddcaAccount: ddcaAccountPda,
          fromMint: mintA.publicKey,
          fromTokenAccount: ddcaFromTokenAccountAddress,
          toMint: mintB.publicKey,
          toTokenAccount: ddcaToTokenAccountAddress,
          operatingAccount: DDCA_OPERATING_ACCOUNT_ADDRESS,
          // hybrid liquidity aggregator accounts
          hlaProgram: HLA_PROGRAM_ADDRESS,
          hlaOperatingAccount: HLA_OPERATING_ACCOUNT_ADDRESS,
          hlaOperatingFromTokenAccount: hlaOperatingFromTokenAccountAddress,
          // system accounts
          operatingFromTokenAccount: ddcaOperatingFromTokenAccountAddress,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          systemProgram: SYSTEM_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
        },
        // signers: [ownerAcc],
        instructions: instructions,
        // hybrid liquidity aggregator specific amm pool accounts
        remainingAccounts: [
          { pubkey: hlaPoolAddress, isWritable: false, isSigner: false },
          { pubkey: hlaProtocolAddress, isWritable: false, isSigner: false },
          { pubkey: hlaAmmAddress, isWritable: false, isSigner: false },
        ],
      }
    );

    await logPostTransactionState("Create", createTx);

    // const accounts = await program.account.vaultAccount.all(ownerAccAddress.toBuffer()); //uncomment to list vault accounts owned by current wallet: ownerAccAddress
    // const accounts = await program.account.vaultAccount.all(anchor.web3.Keypair.generate().publicKey.toBuffer()); //this will fetch empty list
    // console.log(accounts);

    const _ddcaAccount = await program.account.ddcaAccount.fetch(ddcaAccountPda);
    assert.ok(_ddcaAccount.ownerAccAddr.equals(ownerAccountAddress));
    assert.ok((_ddcaAccount.bump = ddcaAccountPdaBump));
    assert.ok((_ddcaAccount.fromTaccAddress = ddcaFromTokenAccountAddress));

    let _ddcaFromTokenAccount = await splTokenAClient.getAccountInfo(ddcaFromTokenAccountAddress);
    assert.ok(_ddcaFromTokenAccount.state === 1);
    assert.ok(_ddcaFromTokenAccount.amount.toNumber() === ddcaFromInitialAmount);
    assert.ok(_ddcaFromTokenAccount.isInitialized);
    assert.ok(_ddcaFromTokenAccount.owner.equals(ddcaAccountPda));
    assert.ok(_ddcaFromTokenAccount.mint.equals(mintA.publicKey));
    

    //UTILS
    async function logPostTransactionState(transactionName, transactionSignature){

      console.log("AFTER %s:", transactionName.toUpperCase());

      // const tx = await provider.connection.getTransaction(transactionSignature, 'recent');
      // console.log(tx);
      
      console.log("  ownerAccountLamports.lamports:     " + (await getLamports(ownerAccountAddress)));
      console.log("  ownerTokenAccount.amount:          " + (await getTokenAAmount(ownerTokenAccountAAddress)));
      console.log();
      console.log("  ddcaAccount.lamports:              " + (await getLamports(ddcaAccountPda)));
      console.log("  ddcaAccount.cli:                   " + "solana account %s", ddcaAccountPda);
      console.log();
      console.log("  ddcaFromTokenAccount.lamports:     " + (await getLamports(ddcaFromTokenAccountAddress)));
      console.log("  ddcaFromTokenAccount.amount:       " + (await getTokenAAmount(ddcaFromTokenAccountAddress)));
      console.log("  ddcaFromTokenAccount.cli (system): " + "solana account %s", ddcaFromTokenAccountAddress);
      console.log("  ddcaFromTokenAccount.cli (token):  " + "spl-token account-info --address %s", ddcaFromTokenAccountAddress);
      console.log();
      console.log("  ddcaToTokenAccount.lamports:       " + (await getLamports(ddcaToTokenAccountAddress)));
      console.log("  ddcaToTokenAccount.amount:         " + (await getTokenBAmount(ddcaToTokenAccountAddress)));
      console.log("  ddcaToTokenAccount.cli (system):   " + "solana account %s", ddcaToTokenAccountAddress);
      console.log("  ddcaToTokenAccount.cli (token):    " + "spl-token account-info --address %s", ddcaToTokenAccountAddress);
      console.log();
      console.log("  transaction:                       " + transactionSignature);
      console.log("  transaction.cli:                   " + "solana confirm %s -v", transactionSignature);
      console.log();
    }

    async function getLamports(accountAddress){
      return (await program.provider.connection.getAccountInfo(accountAddress)).lamports;
    }

    async function getTokenAAmount(tokenAccountAddress){
      return(await splTokenAClient.getAccountInfo(tokenAccountAddress)).amount;
    }

    async function getTokenBAmount(tokenAccountAddress){
      return(await splTokenBClient.getAccountInfo(tokenAccountAddress)).amount;
    }

  });

});

async function createAtaCreateInstructionIfNotExists(ataAddress, mintAddress, ownerAccountAddress, payerAddress, tokenClient) {
  try{
    await tokenClient.getAccountInfo(ataAddress);
    console.log("ATA: %s for mint: %s already exists", ataAddress, mintAddress);
  } catch (err) {
    // INVALID_ACCOUNT_OWNER can be possible if the associatedAddress has
    // already been received some lamports (= became system accounts).
    // Assuming program derived addressing is safe, this is the only case
    // for the INVALID_ACCOUNT_OWNER in this code-path
    if (
      err.message === FAILED_TO_FIND_ACCOUNT ||
      err.message === INVALID_ACCOUNT_OWNER
    ) {
      console.log("ATA: %s for mint: %s was not found (%s). Generating 'create' instruction...", ataAddress, mintAddress, err.message);
      let [_, ataCreateInstruction] = 
      await createAtaCreateInstruction(ataAddress, mintAddress, ownerAccountAddress, payerAddress);
      return ataCreateInstruction;
    } else {
      throw err;
    }
  }
}

async function createAtaCreateInstruction(ataAddress, mintAddress, ownerAccountAddress, payerAddress) {
  if(ataAddress === null){
    ataAddress = await Token.getAssociatedTokenAddress(
      associatedProgramId=ASSOCIATED_TOKEN_PROGRAM_ID,
      programId=TOKEN_PROGRAM_ID,
      mintAddress=mintAddress,
      owner=ownerAccountAddress,
    );
  }
  const ataCreateInstruction = new anchor.web3.Transaction();
  ataCreateInstruction.add(Token.createAssociatedTokenAccountInstruction(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    mintAddress,
    ataAddress,
    ownerAccountAddress,
    payerAddress,
  ));
  return [ataAddress, ataCreateInstruction];
}
