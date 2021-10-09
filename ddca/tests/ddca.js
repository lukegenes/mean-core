const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { SystemProgram } = anchor.web3;
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, Token } = require("@solana/spl-token");

describe("ddca", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  //
  const SYSTEM_PROGRAM_ID = anchor.web3.SystemProgram.programId;
  const program = anchor.workspace.Ddca;
  const payer = anchor.web3.Keypair.generate();
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

  it("One-time test setup", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );

    mintA = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    ownerTokenAccountAAddress = await mintA.createAccount(provider.wallet.publicKey);

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
      0,
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
        provider.wallet.publicKey.toBuffer(),
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

    let ownerAccountLamports = (
      await program.provider.connection.getAccountInfo(
        program.provider.wallet.publicKey
      )
    ).lamports;

    console.log("TEST PARAMETERS:")
    console.log("  Program ID:                         " + program.programId);
    console.log("  mintAuthority.publicKey:            " + mintAuthority.publicKey);
    console.log("  mintA.publicKey:                    " + mintA.publicKey);
    console.log("  mintB.publicKey:                    " + mintB.publicKey);
    console.log("  ownerAccountAddress:                " + provider.wallet.publicKey);
    console.log("  ownerAccount.lamports:              " + ownerAccountLamports);
    console.log("  ownerFromTokenAccountAddress:       " + ownerTokenAccountAAddress);
    console.log("  blockHeight:                        " + blockHeight);
    console.log("  ddcaAccountPda:                     " + ddcaAccountPda);
    console.log("  ddcaAccountPdaBump:                 " + ddcaAccountPdaBump);
    console.log("  ddcaFromTokenAccountAddress:        " + ddcaFromTokenAccountAddress);
    console.log("  SYSTEM_PROGRAM_ID:                  " + SYSTEM_PROGRAM_ID);
    console.log("  TOKEN_PROGRAM_ID:                   " + TOKEN_PROGRAM_ID);
    console.log();
    
    const createTx = await program.rpc.createDdca(new anchor.BN(blockHeight), ddcaAccountPdaBump,
      new anchor.BN(ddcaFromInitialAmount), new anchor.BN(ddcaFromAmountPerSwap), new anchor.BN(ddcaIntervalInSeconds), 
      {
        accounts: {
          ownerAccount: provider.wallet.publicKey,
          ownerFromTokenAccount: ownerTokenAccountAAddress,
          ddcaAccount: ddcaAccountPda,
          fromMint: mintA.publicKey,
          fromTokenAccount: ddcaFromTokenAccountAddress,
          toMint: mintB.publicKey,
          toTokenAccount: ddcaToTokenAccountAddress,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          systemProgram: SYSTEM_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
        },
      }
    );

    await logPostTransactionState("Create", createTx);

    // const accounts = await program.account.vaultAccount.all(provider.wallet.publicKey.toBuffer()); //uncomment to list vault accounts owned by current wallet: provider.wallet.publicKey
    // const accounts = await program.account.vaultAccount.all(anchor.web3.Keypair.generate().publicKey.toBuffer()); //this will fetch empty list
    // console.log(accounts);

    const _ddcaAccount = await program.account.ddcaAccount.fetch(ddcaAccountPda);
    assert.ok(_ddcaAccount.ownerAccAddr.equals(provider.wallet.publicKey));
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

      // const tx = await provider.connection.getTransaction(transactionSignature, 'confirmed');
      // console.log(tx);
      
      console.log("  ownerAccountLamports.lamports:     " + (await getLamports(program.provider.wallet.publicKey)));
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
