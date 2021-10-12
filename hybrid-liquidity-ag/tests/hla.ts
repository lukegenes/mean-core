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

  it('No pools found', async () => {
    
    try {
      // Add your test here.
      const amount = 5000000000;
      const slippage = 1;
      const provider = anchor.getProvider();
      const wallet = provider.wallet;
      const program = anchor.workspace.Hla;
      const tx = await program.rpc.swap(
        new BN(amount),
        slippage,
        {
          accounts: {
            owner: wallet.publicKey,
            fromAccount: PublicKey.default,
            fromToken: PublicKey.default,
            toAccount: PublicKey.default,
            toToken: PublicKey.default
          },
          signers: []
        }
      );
      
      assert.ok(false);

    } catch (err) {
      const errMsg = "No pools found";
      assert.equal(err.toString(), errMsg);
      assert.equal(err.msg, errMsg);
      assert.equal(err.code, 301);
    }

  });

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

    const ownerFromAccount = await spl.Token.getAssociatedTokenAddress(
      spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      spl.TOKEN_PROGRAM_ID,
      USDC,
      wallet.publicKey
    );

    const ownerToAccount = await spl.Token.getAssociatedTokenAddress(
      spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      spl.TOKEN_PROGRAM_ID,      
      USDT,
      wallet.publicKey
    );

    const tx = await program.rpc.swap(
      new BN(amount),
      slippage,
      {
        accounts: {
          owner: wallet.publicKey,
          fromAccount: ownerFromAccount,
          fromToken: USDC,
          toAccount: ownerToAccount,
          toToken: USDT
        },
        signers: []
      }
    );
    
    assert.ok(tx);

    } catch (err) {
      console.log(err);
    }
  });
});
