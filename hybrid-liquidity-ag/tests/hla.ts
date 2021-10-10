const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { PublicKey } = anchor.web3;
const { BN } = anchor;

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

  it('swapped', async () => {
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
    
    assert.ok(tx);    
  });
});
