# Mean Protocol

**Mean Core** is the repository for the **Mean Protocol** Smart Contracts (Programs) in Solana.
The Mean Protocol is maintained by the Mean DAO and is deployed on the Solana Blockchain. The protocol facilitates transaction coordination using several sub-graphs and programs, such as the Universal Liquidity (ULI) and the Money Streaming (MSP) programs. To explore all the different components of the Mean Protocol, head over to the [Developers Page](https://docs.meanfi.com/platform/developers).

## Getting Started

* The **Mean Protocol** is in active development and the programs are subject to change
* For detailed documentation, please read the Developer Docs 👉 https://docs.meanfi.com/platform/developers

An example of a web3 Dapp implementing the different programs in the Mean Protocol is [MeanFi](https://meanfi.com).

To leverage the Mean Protocol Programs in your own dapp, go to the [Mean Protocol SDK repo](https://github.com/mean-dao/mean-sdk), and follow the instructions there.

## Program Catalog

| Program | Description | Version
| :-- | :-- | :--|
| `money-streaming` | Implementation of the **[Money Streaming Protocol](https://docs.meanfi.com/platform/specifications/money-streaming-protocol)** | **1.1.0** |
| `ddca` | Implementation of the **[DDCA Protocol]()** | **--** |
| `hybrid-liquidity-ag` | Implementation of the **[Hybrid Liquidity Aggregator Protocol]()** | **--**


### Money Streaming

Money streaming represents the idea of continuous payments over time. Block numbers are used to measure time in the blockchain and continuously update the balances of the parties in the contract. Head over to the **[Whitepaper](https://docs.meanfi.com/platform/specifications/money-streaming-protocol)** for more details.

The Money Streaming Program is an implementation of the protocol built and deployed on the Solana blockchain, with address `H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko` on [Mainnet Beta](https://explorer.solana.com/address/H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko).

### Decentralized DCA

Decentralized Dollar Cost Averaging (DDCA) is great to help people become regular investors every day. Decentralized DCA gives account owners a way to automate their investment strategy without the dependence on a centralized entity like Coinbase or Robinhood.

The DDCA Program is an implementation of the **[DDCA Protocol]()** on the Solana blockchain.


### Hybrid Liquidity Aggregator

The Hybrid Liquidity Aggregator is a phenomenal tool to help your users access massive liquidity from multiple protocols and optimize their routing, fees, slippage, and pricing impact. This is useful if you want to guarantee the best swap prices across multiple Automated Market Makers (AMMs) and Serum's CLOB without having to manually implement each of them. One program to rule them all.

The HLA Program is an implementation of the **[Hybrid Liquidity Aggregator Protocol]()** on the Solana blockchain. It currently supports Serum's CLOB and AMMs from Raydium, Orca, Saber and Mercurial.
