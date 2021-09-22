# Mean Core

**Mean Core** is the repository for the **Mean Protocol** Smart Contracts (Programs) in Solana.

<h2>Programs</h2>

| Program | Description | Version
| :-- | :-- | :--|
| `money-streaming` | Rust program to implement the **[Money Streaming Protocol](https://docs.google.com/document/d/19W5V2B8eyFIocccgSP4orn6Wi1El07LQSyaT7yw6hMQ)** | **1.1.0** |
| `ddca` | Rust program to implement the **[DDCA Protocol]()** | **--** |
| `hybrid-liquidity-ag` | Rust program to implement the **[Universal Liquidity Aggregator Protocol]()** | **--**

<h2>Note</h2>

* The **Mean Protocol** is in active development and the programs are subject to change

<br/>
<h2>Money Streaming</h2>

The **Money Streaming** program is an implementation of the **[Money Streaming Protocol](https://docs.google.com/document/d/19W5V2B8eyFIocccgSP4orn6Wi1El07LQSyaT7yw6hMQ)** built and deployed on the **Solana** blockchain, with address `H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko` on [Mainnet Beta](https://explorer.solana.com/address/H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko).

Money streaming represents the idea of continuous payments over time. Block numbers are used to measure time in the blockchain and continuously update the balances of the parties in the contract.

Looking for a Dapp that leverages the Money Streaming Protocol?
* See MeanFi at https://app.meanfi.com
* See MeanFi repo: https://github.com/mean-dao/meanfi-ui

<br/>
<h2>DDCA</h2>

**DDCA** (Decentralized Dollar Cost Averaging) is an implementation of the **[DDCA Protocol]()** on the **Solana** blockchain.
The **DDCA** showcases the idea of an on-chain DCA investment strategy without a central timekeeper, such as a **Centralized Exchange** or centrally managed application.

<br/>
<h2>Hybrid Liquidity Aggregator</h2>

**Hybrid Liquidity Aggregator** is an implementation of the **[Universal Liquidity Aggregator Protocol]()** on the **Solana** blockchain.
The **Hybrid Liquidity Aggregator** aggregates multiple **Automated Market Makers** (AMMs) built on the Solana blockchain to leverage their provided **Liquidity Pools** (LP) to offer the best possible exchange rates between the supported token pairs

Supported AMMs:

* **Raydium**
* **Orca**
* **Saber HQ**
