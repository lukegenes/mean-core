/**
 * Solana
 */
import { PublicKey } from "@solana/web3.js"

declare global {
    export interface String {
        toPublicKey(): PublicKey;
    }
}

/**
 * MSP Instructions types
 */
export const enum MSP_ACTIONS {
    oneTimePayment = 1,
    scheduleOneTimePayment = 2,
    createStream = 1,
    createStreamWithFunds = 2,
    addFunds = 2,
    withdraw = 3,
    pauseStream = 4,
    resumeStream = 5,
    proposeUpdate = 6,
    answerUpdate = 7,
    createTreasury = 8,
    closeStream = 9
}

/**
 * Transaction fees
 */
export type TransactionFees = {
    /* Solana fees calculated based on the tx signatures and cluster */
    blockchainFee: number;
    /* MSP flat fee amount depending of the instruction that is being executed */
    mspFlatFee: number;
    /* MSP fee amount in percent depending of the instruction that is being executed */
    mspPercentFee: number;
}

/**
 * Transaction fees parameters
 */
export type TransactionFeesParams = {
    instruction: MSP_ACTIONS;
    signaturesAmount: number;
}

/**
 * Transaction message
 */
export type TransactionMessage = {
    action: string,
    description: string,
    amount: number,
    fees: TransactionFees
}

/**
 * Stream activity
 */
export type StreamActivity = {
    signature: string,
    initializer: string,
    action: string;
    amount: number;
    mint: string;
    blockTime: number;
    utcDate: string;
}

/**
 * Treasury info
 */
export type TreasuryInfo = {
    id: PublicKey | string | undefined,
    initialized: boolean,
    treasuryBlockHeight: number,
    treasuryMintAddress: PublicKey | string | undefined,
    treasuryBaseAddress: PublicKey | string | undefined,
}

/**
 * Stream contract terms
 */
export type StreamTermsInfo = {
    id: PublicKey | string | undefined,
    initialized: boolean,
    streamId: PublicKey | string | undefined,
    streamMemo: String,
    treasurerAddress: PublicKey | string | undefined,
    beneficiaryAddress: PublicKey | string | undefined,
    associatedToken: PublicKey | string | undefined,
    rateAmount: number,
    rateIntervalInSeconds: number,
    rateCliffInSeconds: number,
    cliffVestAmount: number,
    cliffVestPercent: number,
    autoPauseInSeconds: number
}

/**
 * Stream info
 */
export type StreamInfo = {
    id: PublicKey | string | undefined,
    initialized: boolean,
    memo: String,
    treasurerAddress: PublicKey | string | undefined,
    rateAmount: number,
    rateIntervalInSeconds: number,
    fundedOnUtc: Date | string | undefined,
    startUtc: Date | string | undefined,
    rateCliffInSeconds: number,
    cliffVestAmount: number,
    cliffVestPercent: number,
    beneficiaryAddress: PublicKey | string | undefined,
    associatedToken: PublicKey | string | undefined,
    escrowVestedAmount: number,
    escrowUnvestedAmount: number,
    treasuryAddress: PublicKey | string | undefined,
    escrowEstimatedDepletionUtc: Date | string | undefined,
    totalDeposits: number,
    totalWithdrawals: number,
    escrowVestedAmountSnap: number,
    escrowVestedAmountSnapBlockHeight: number,
    escrowVestedAmountSnapBlockTime: number,
    streamResumedBlockHeight: number,
    streamResumedBlockTime: number,
    autoPauseInSeconds: number,
    isStreaming: boolean,
    isUpdatePending: boolean,
    transactionSignature: string | undefined,
    blockTime: number,
}

export class Constants {

    static MSP_PROGRAM_KEY = new PublicKey('9yMq7x4LstWYWi14pr8BEBsEX33L3HnugpiM2PT96x4k'); //'37z61WhJCAaDADwcpJRHgr66FUhHB9TfkS49Ssvp3Cdb';
    static MSP_OPS_KEY = new PublicKey('BgxJuujLZDR27SS41kYZhsHkXx6CP2ELaVyg1qBxWYNU');
    static MEMO_PROGRAM_KEY = new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr');
    static ASSOCIATED_TOKEN_PROGRAM_KEY = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');
    static WSOL_TOKEN_MINT_KEY = new PublicKey('So11111111111111111111111111111111111111112');
    static USDC_TOKEN_MINT_KEY = new PublicKey('AbQBt9V212HpPVk64YWAApFJrRzdAdu66fwF9neYucpU'); //'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';
    static USDT_TOKEN_MINT_KEY = new PublicKey('42f2yFqXh8EDCRCiEBQSweWqpTzKGa9DC8e7UjUfFNrP'); //'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB';
    static ETH_TOKEN_MINT_KEY = new PublicKey('2FPyTwcZLUg1MDrwsyoP4D6s1tM7hAkHYRjkNb5w6Pxk');
    static SERUM_DEX_KEY = new PublicKey('9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin');
    static SERUM_SWAP_KEY = new PublicKey('22Y43yTVxuUkoRKdm9thyRhQ3SdgQS7c7kB6UNCiaczD');
    static DEVNET_CLUSTER = 'https://api.devnet.solana.com';
}
