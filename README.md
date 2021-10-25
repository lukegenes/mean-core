# Mean Protocol

The Mean Protocol is a set of rules and interoperable smart contracts that help application developers facilitate everyday banking workflows and investment banking operations.  

The Mean Protocol is maintained by the Mean DAO and is deployed on the Solana Blockchain. The protocol facilitates transaction coordination using several sub-graphs and programs, such as the Hybrid Liquidity Aggregator, DDCA, and Money Streaming programs. To explore all the different components of the Mean Protocol, head over to the [Developers Page](https://docs.meanfi.com/platform/developers).

## Getting Started

* The **Mean Protocol** is in active development and the programs are subject to change
* For detailed documentation, please read the Developer Docs 👉 https://docs.meanfi.com/platform/developers

### Related Repos
![Mean Repos](https://user-images.githubusercontent.com/714487/138731452-a87355e0-5579-4da9-bb12-3aa90c526a8c.png)
- Repo for Mean Protocol SDKs 👉 **[HERE](https://github.com/mean-dao/mean-sdk)** (how to use instructions there)
- Repo for MeanFi UI 👉 **[HERE](https://github.com/mean-dao/meanfi-ui)** 
[MeanFi](https://meanfi.com) is a web3 app implementing the different programs in the Mean Protocol 

## Program Catalog

| Program | Description
| :-- | :-- |
| `money-streaming` | Implementation of the **[Money Streaming Protocol](https://docs.meanfi.com/platform/specifications/money-streaming-protocol)**
| `ddca` | Implementation of the **[DDCA Protocol]()**
| `hybrid-liquidity-ag` | Implementation of the **[Hybrid Liquidity Aggregator Protocol]()**


### Money Streaming

Money streaming represents the idea of continuous payments over time. Block numbers are used to measure time in the blockchain and continuously update the balances of the parties in the contract. Head over to the **[Whitepaper](https://docs.meanfi.com/platform/specifications/money-streaming-protocol)** for more details.

The Money Streaming Program is an implementation of the protocol built and deployed on the Solana blockchain, with address `H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko` on [Mainnet Beta](https://explorer.solana.com/address/H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko).

### Decentralized DCA

Decentralized Dollar Cost Averaging (DDCA) is great to help people become regular investors every day. Decentralized DCA gives account owners a way to automate their investment strategy without the dependence on a centralized entity like Coinbase or Robinhood.

The DDCA Program is an implementation of the **[DDCA Protocol]()** on the Solana blockchain with address `3nmm1awnyhABJdoA25MYVksxz1xnpUFeepJJyRTZfsyD` on [Mainnet Beta](https://explorer.solana.com/address/3nmm1awnyhABJdoA25MYVksxz1xnpUFeepJJyRTZfsyD).


### Hybrid Liquidity Aggregator

The Hybrid Liquidity Aggregator (HLA) is a phenomenal tool to help your users access massive liquidity from multiple protocols and optimize their routing, fees, slippage, and pricing impact. This is useful if you want to guarantee the best swap prices across multiple Automated Market Makers (AMMs) and Serum's CLOB without having to manually implement each of them. One program to rule them all.

The HLA Program is an implementation of the **[Hybrid Liquidity Aggregator Protocol]()** on the Solana blockchain. It currently supports Serum's CLOB and the Raydium, Orca, Saber and Mercurial AMMs.

The HLA has 2 parts:
- A client side aggregator (you can find it on the Mean SDK repo)
- An on-chain aggregator (found on this repo, WIP) with address `B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3` on [Mainnet Beta](https://explorer.solana.com/address/B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3).
