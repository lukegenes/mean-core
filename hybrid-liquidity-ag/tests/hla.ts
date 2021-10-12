const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { PublicKey } = anchor.web3;
const { BN } = anchor;
const spl = require("@solana/spl-token");


describe('hla', async () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it('initialized', async () => {
    const provider = anchor.getProvider();
    const wallet = provider.wallet;
    const tx = await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(wallet.publicKey, 1000000000),
      "confirmed"
    );
    assert.ok(tx);
  });

  // it('No pools found', async () => {
    
  //   try {
  //     const amount = 5000000000;
  //     const slippage = 1;
  //     const provider = anchor.getProvider();
  //     const wallet = provider.wallet;
  //     const program = anchor.workspace.Hla;
  //     const tx = await program.rpc.swap({
  //       ctx: {
  //         accounts: {
  //           feePayer: wallet.publicKey,
  //           poolAccount: PublicKey.default,
  //           protocolAccount: PublicKey.default,
  //           ammAccount: PublicKey.default,
  //           vaultAccount: PublicKey.default,
  //           fromTokenMint: PublicKey.default,
  //           fromTokenAccount: PublicKey.default,
  //           toTokenMint: PublicKey.default,
  //           toTokenAccount: PublicKey.default,
  //         },
  //         signers: []
  //       },
  //       fromAmount: new BN(amount),
  //       minOutAmount: new BN(amount * (100 - slippage) / 100),
  //       slippage: slippage
  //     });
      
  //     assert.ok(false);

  //   } catch (err) {
  //     const errMsg = "No pools found";
  //     assert.equal(err.toString(), errMsg);
  //     assert.equal(err.msg, errMsg);
  //     assert.equal(err.code, 301);
  //   }

  // });

  it('swapped', async () => {
    try {
      // Add your test here.
      const amount = 5000000000;
      const slippage = 1;
      const provider = anchor.getProvider();
      const wallet = provider.wallet;
      const program = anchor.workspace.Hla;
      const USDC = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
      const USDT = new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
      const slot = await provider.connection.getSlot("confirmed");
      const { blockTime } = await provider.connection.getBlock(slot, { commitment: "confirmed" });

      const ddcaAccount = await PublicKey.createProgramAddress(
        [
          Buffer.from(Uint8Array.from([blockTime]))
        ],
        program.programId
      );

      const minBalance = await provider.connection.getMinimumBalanceForRentExemption(0);
      let tx = await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(ddcaAccount, minBalance),
        "confirmed"
      );

      const ownerFromAccount = await spl.Token.getAssociatedTokenAddress(
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        spl.TOKEN_PROGRAM_ID,
        USDC,
        wallet.publicKey,
        true
      );

      const ownerToAccount = await spl.Token.getAssociatedTokenAddress(
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        spl.TOKEN_PROGRAM_ID,      
        USDT,
        wallet.publicKey,
        true
      );

      const hlaOpsAccount = new PublicKey("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
      const hlaOpsTokenAccount = await spl.Token.getAssociatedTokenAddress(
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        spl.TOKEN_PROGRAM_ID,      
        USDC,
        hlaOpsAccount,
        true
      );

      const poolAccount = new PublicKey("2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf");
      const protocolAccount = new PublicKey("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ");
      const ammAccount = new PublicKey("YAkoNb6HKmSxQN9L8hiBE5tPJRsniSSMzND1boHmZxe");
      const [swapAuthority] = PublicKey.findProgramAddress(
        [wallet.publicKey.toBuffer()],
        program.programId
      );

      tx = await program.rpc.swap({
        ctx: {
          accounts: {
            feePayer: wallet.publicKey,
            // poolAccount: new PublicKey("2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf"),
            // protocolAccount: new PublicKey("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ"),
            // ammAccount: new PublicKey("YAkoNb6HKmSxQN9L8hiBE5tPJRsniSSMzND1boHmZxe"),
            vaultAccount: ddcaAccount,
            fromTokenMint: USDC,
            fromTokenAccount: ownerFromAccount,
            toTokenMint: USDT,
            toTokenAccount: ownerToAccount,
            hlaOpsAccount: hlaOpsAccount,
            hlaOpsTokenAccount: hlaOpsTokenAccount
          },
          signers: [],
          remainingAccounts: [
            { pubkey: , isSigner: false, isWritable: false  }
          ]
        },
        fromAmount: new BN(amount),
        minOutAmount: new BN(amount * (100 - slippage) / 100),
        slippage: slippage
      });
      
      assert.ok(tx);

    } catch (err) {
      console.log(err);
    }
  });
});
