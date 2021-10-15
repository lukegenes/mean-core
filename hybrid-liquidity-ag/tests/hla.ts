const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { PublicKey } = anchor.web3;
const { BN } = anchor;
const spl = require("@solana/spl-token");

const ANCHOR_PROVIDER_URL= "https://solana-api.projectserum.com";

const tests = async () => {
  try {
    const registry_idl = JSON.parse(require('fs').readFileSync('./target/idl/hla.json', 'utf8'));
    const programId = new anchor.web3.PublicKey('B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3');
    const registry = new anchor.Program(registry_idl, programId);
    // Add your test here.
    const amount = 5000000000;
    const slippage = 1;
    console.log('anchor.Provider.env()', anchor.Provider.env());
    anchor.setProvider(anchor.Provider.env());
    const provider = anchor.getProvider();
    const wallet = provider.wallet;
    const program = anchor.workspace.Hla;
    // const USDC = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    // const USDT = new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
    const USDC = new PublicKey("2tWC4JAdL4AxEFJySziYJfsAnW2MHKRo98vbAPiRDSk8");
    const USDT = new PublicKey("EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS");

    const slot = await provider.connection.getSlot("confirmed");
    const { blockTime } = await provider.connection.getBlock(slot, { commitment: "confirmed" });

    const ddcaAccount = await PublicKey.createProgramAddress(
      [
        Buffer.from(Uint8Array.from([blockTime]))
      ],
      program.programId
    );

    const minBalance = await registry.get.getMinimumBalanceForRentExemption(0);
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

    const poolAccount = new PublicKey("YakofBo4X3zMxa823THQJwZ8QeoU8pxPdFdxJs7JW57");
    const protocolAccount = new PublicKey("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ");
    const ammAccount = new PublicKey("VeNkoB1HvSP6bSeGybQDnx9wTWFsQb2NBCemeCDSuKL");

    const [swapAuthority] = PublicKey.findProgramAddress(
      [wallet.publicKey.toBuffer()],
      program.programId
    );

    tx = await registry.rpc.swap({
      ctx: {
        accounts: {
          feePayer: wallet.publicKey,
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
          { pubkey: poolAccount, isSigner: false, isWritable: false },
          { pubkey: protocolAccount, isSigner: false, isWritable: false },
          { pubkey: ammAccount, isSigner: false, isWritable: false },
          { pubkey: swapAuthority, isSigner: true, isWritable: false },
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
}

tests().then(() => {});

// describe('hla', async () => {

//   // Configure the client to use the local cluster.
//   anchor.setProvider(anchor.Provider.env());

//   it('initialized', async () => {
//     const provider = anchor.getProvider();
//     const wallet = provider.wallet;
//     const tx = await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(wallet.publicKey, 1000000000),
//       "confirmed"
//     );
//     assert.ok(tx);
//   });

//   it('No pools found', async () => {
    
//     try {
//       const amount = 5000000000;
//       const slippage = 1;
//       const provider = anchor.getProvider();
//       const wallet = provider.wallet;
//       const program = anchor.workspace.Hla;

//       const USDC = new PublicKey("2tWC4JAdL4AxEFJySziYJfsAnW2MHKRo98vbAPiRDSk8");
//       const USDT = new PublicKey("EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS");
//       const slot = await provider.connection.getSlot("confirmed");
//       const { blockTime } = await provider.connection.getBlock(slot, { commitment: "confirmed" });

//       const ddcaAccount = await PublicKey.createProgramAddress(
//         [
//           Buffer.from(Uint8Array.from([blockTime]))
//         ],
//         program.programId
//       );

//       const minBalance = await provider.connection.getMinimumBalanceForRentExemption(0);
//       let tx = await provider.connection.confirmTransaction(
//         await provider.connection.requestAirdrop(ddcaAccount, minBalance),
//         "confirmed"
//       );

//       const ownerFromAccount = await spl.Token.getAssociatedTokenAddress(
//         spl.ASSOCIATED_TOKEN_PROGRAM_ID,
//         spl.TOKEN_PROGRAM_ID,
//         USDC,
//         wallet.publicKey,
//         true
//       );

//       const ownerToAccount = await spl.Token.getAssociatedTokenAddress(
//         spl.ASSOCIATED_TOKEN_PROGRAM_ID,
//         spl.TOKEN_PROGRAM_ID,      
//         USDT,
//         wallet.publicKey,
//         true
//       );

//       const hlaOpsAccount = new PublicKey("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
//       const hlaOpsTokenAccount = await spl.Token.getAssociatedTokenAddress(
//         spl.ASSOCIATED_TOKEN_PROGRAM_ID,
//         spl.TOKEN_PROGRAM_ID,      
//         USDC,
//         hlaOpsAccount,
//         true
//       );

//       const poolAccount = new PublicKey("2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf");
//       const protocolAccount = new PublicKey("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ");
//       const ammAccount = new PublicKey("YAkoNb6HKmSxQN9L8hiBE5tPJRsniSSMzND1boHmZxe");

//       const [swapAuthority] = PublicKey.findProgramAddress(
//         [wallet.publicKey.toBuffer()],
//         program.programId
//       );

//       const tx2 = await program.rpc.swap({
//         ctx: {
//           accounts: {
//             feePayer: wallet.publicKey,
//             vaultAccount: ddcaAccount,
//             fromTokenMint: USDC,
//             fromTokenAccount: ownerFromAccount,
//             toTokenMint: USDT,
//             toTokenAccount: ownerToAccount,
//             hlaOpsAccount: hlaOpsAccount,
//             hlaOpsTokenAccount: hlaOpsTokenAccount
//           },
//           signers: [],
//           remainingAccounts: [
//             { pubkey: poolAccount, isSigner: false, isWritable: false },
//             { pubkey: protocolAccount, isSigner: false, isWritable: false },
//             { pubkey: ammAccount, isSigner: false, isWritable: false },
//             { pubkey: swapAuthority, isSigner: true, isWritable: false },
//           ]
//         },
//         fromAmount: new BN(amount),
//         minOutAmount: new BN(amount * (100 - slippage) / 100),
//         slippage: slippage
//       });
      
//       assert.ok(false);

//     } catch (err) {
//       const errMsg = "No pools found";
//       assert.strictEqual
//       assert.equal(err.toString(), errMsg);
//     }

//   });

//   it('swapped', async () => {
//     try {
//       // Add your test here.
//       const amount = 5000000000;
//       const slippage = 1;
//       const provider = anchor.getProvider();
//       const wallet = provider.wallet;
//       const program = anchor.workspace.Hla;
//       // const USDC = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
//       // const USDT = new PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
//       const USDC = new PublicKey("2tWC4JAdL4AxEFJySziYJfsAnW2MHKRo98vbAPiRDSk8");
//       const USDT = new PublicKey("EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS");

//       const slot = await provider.connection.getSlot("confirmed");
//       const { blockTime } = await provider.connection.getBlock(slot, { commitment: "confirmed" });

//       const ddcaAccount = await PublicKey.createProgramAddress(
//         [
//           Buffer.from(Uint8Array.from([blockTime]))
//         ],
//         program.programId
//       );

//       const minBalance = await provider.connection.getMinimumBalanceForRentExemption(0);
//       let tx = await provider.connection.confirmTransaction(
//         await provider.connection.requestAirdrop(ddcaAccount, minBalance),
//         "confirmed"
//       );

//       const ownerFromAccount = await spl.Token.getAssociatedTokenAddress(
//         spl.ASSOCIATED_TOKEN_PROGRAM_ID,
//         spl.TOKEN_PROGRAM_ID,
//         USDC,
//         wallet.publicKey,
//         true
//       );

//       const ownerToAccount = await spl.Token.getAssociatedTokenAddress(
//         spl.ASSOCIATED_TOKEN_PROGRAM_ID,
//         spl.TOKEN_PROGRAM_ID,      
//         USDT,
//         wallet.publicKey,
//         true
//       );

//       const hlaOpsAccount = new PublicKey("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
//       const hlaOpsTokenAccount = await spl.Token.getAssociatedTokenAddress(
//         spl.ASSOCIATED_TOKEN_PROGRAM_ID,
//         spl.TOKEN_PROGRAM_ID,      
//         USDC,
//         hlaOpsAccount,
//         true
//       );

//       const poolAccount = new PublicKey("YakofBo4X3zMxa823THQJwZ8QeoU8pxPdFdxJs7JW57");
//       const protocolAccount = new PublicKey("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ");
//       const ammAccount = new PublicKey("VeNkoB1HvSP6bSeGybQDnx9wTWFsQb2NBCemeCDSuKL");

//       const [swapAuthority] = PublicKey.findProgramAddress(
//         [wallet.publicKey.toBuffer()],
//         program.programId
//       );

//       tx = await program.rpc.swap({
//         ctx: {
//           accounts: {
//             feePayer: wallet.publicKey,
//             vaultAccount: ddcaAccount,
//             fromTokenMint: USDC,
//             fromTokenAccount: ownerFromAccount,
//             toTokenMint: USDT,
//             toTokenAccount: ownerToAccount,
//             hlaOpsAccount: hlaOpsAccount,
//             hlaOpsTokenAccount: hlaOpsTokenAccount
//           },
//           signers: [],
//           remainingAccounts: [
//             { pubkey: poolAccount, isSigner: false, isWritable: false },
//             { pubkey: protocolAccount, isSigner: false, isWritable: false },
//             { pubkey: ammAccount, isSigner: false, isWritable: false },
//             { pubkey: swapAuthority, isSigner: true, isWritable: false },
//           ]
//         },
//         fromAmount: new BN(amount),
//         minOutAmount: new BN(amount * (100 - slippage) / 100),
//         slippage: slippage
//       });
      
//       assert.ok(tx);

//     } catch (err) {
//       console.log(err);
//     }
//   });
// });
