const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, Token } = require("@solana/spl-token");

describe('hla', async() => {

  const provider = anchor.Provider.local();
  anchor.setProvider(provider);  
  const wallet = provider.wallet;
  const program = anchor.workspace.Hla;

  it ('swapped', async() => {
    const amount = 5000000000;
    const slippage = 1; 
    const USDC = new anchor.web3.PublicKey("2tWC4JAdL4AxEFJySziYJfsAnW2MHKRo98vbAPiRDSk8");
    const USDT = new anchor.web3.PublicKey("EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS");
    const slot = await provider.connection.getSlot("confirmed");
    const blockTime = await provider.connection.getBlockTime(slot);
    const blockTimeBytes = new anchor.BN(blockTime).toBuffer('le', 8);
    const minBalance = await provider.connection.getMinimumBalanceForRentExemption(0);
    
    const ddcaAccountSeed = [
      wallet.publicKey.toBuffer(),
      blockTimeBytes,
      Buffer.from(anchor.utils.bytes.utf8.encode("hla-seed")),
    ];

    const [ddcaAccount] = await anchor.web3.PublicKey.findProgramAddress(
      ddcaAccountSeed,
      program.programId
    );

    const ownerFromAccount = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      USDC,
      ddcaAccount,
      true
    );

    const ownerToAccount = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,      
      USDT,
      ddcaAccount,
      true
    );

    const hlaOpsAccount = new anchor.web3.PublicKey("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
    const hlaOpsTokenAccount = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,      
      USDC,
      hlaOpsAccount,
      true
    );

    const poolAccount = new anchor.web3.PublicKey("YakofBo4X3zMxa823THQJwZ8QeoU8pxPdFdxJs7JW57");
    const protocolAccount = new anchor.web3.PublicKey("SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ");
    const ammAccount = new anchor.web3.PublicKey("VeNkoB1HvSP6bSeGybQDnx9wTWFsQb2NBCemeCDSuKL");

    const [swapAuthority, seed] = await anchor.web3.PublicKey.findProgramAddress(
      [
        ddcaAccount.toBuffer(),
        Buffer.from(Uint8Array.from([blockTime]))
      ],
      program.programId
    );

    const tx = await program.rpc.swap(
      new anchor.BN(amount),
      new anchor.BN((amount * (100 - slippage)) / 100),
      slippage,
      {
        accounts: {
          vaultAccount: ddcaAccount,
          fromTokenMint: USDC,
          fromTokenAccount: ownerFromAccount,
          toTokenMint: USDT,
          toTokenAccount: ownerToAccount,
          hlaOpsAccount: hlaOpsAccount,
          hlaOpsTokenAccount: hlaOpsTokenAccount,
          tokenProgramAccount: TOKEN_PROGRAM_ID,
        },
        remainingAccounts: [
          { pubkey: poolAccount, isSigner: false, isWritable: false },
          { pubkey: protocolAccount, isSigner: false, isWritable: false },
          { pubkey: ammAccount, isSigner: false, isWritable: false },
          { pubkey: swapAuthority, isSigner: false, isWritable: false },
        ]
      }
    );
    
    assert.ok(tx);
  });
})

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
