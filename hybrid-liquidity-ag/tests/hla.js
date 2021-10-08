const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { SystemProgram, PublicKey } = anchor.web3;
const bn = require("bn.js");

describe('mysolanaapp', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Hla;
  const owner = anchor.web3.Keypair.generate();

  it('Is initialized', async () => {

    console.log('init');
    const tx = await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(owner.publicKey, 1000000000),
      "confirmed"
    );
    const ownerBalance = await provider.connection.getBalance(owner.publicKey);
    assert.strictEqual(ownerBalance, 1000000000, 'balance ok');

  });

  it('Is swapped!', async () => {
    const program = anchor.workspace.Mysolanaapp;
    const tx = await program.rpc.initialize();
    console.log("Your transaction signature", tx);
  });
});

describe('hla', async () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Hla;
  const owner = anchor.web3.Keypair.generate();

  it('Init state', async () => {

    console.log('init');
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(owner.publicKey, 1000000000),
      "confirmed"
    );
    const ownerBalance = await provider.connection.getBalance(owner.publicKey);
    assert.strictEqual(ownerBalance, 1000000000, 'balance ok');

  });

  it('it swapped!', async () => {
    // Add your test here.

    console.log('swap');
    const amount = 5;
    const slippage = 1;

    const tx = await program.rpc.swap({
      amountIn: amount,
      slippage,
      accounts: {
        owner: owner.publicKey,
        fromAccount: PublicKey.default,
        fromToken: PublicKey.default,
        toAccount: PublicKey.default,
        toToken: [
          PublicKey.default,
          PublicKey.default
        ]
      },
      signers: [owner],
    });
    
    console.log("Your transaction signature", tx);
    
  });
});
