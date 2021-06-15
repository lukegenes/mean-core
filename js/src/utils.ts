import { BN } from "@project-serum/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { Constants } from "./constants";
import { Layout } from "./layout";
import { StreamInfo } from "./money-streaming";
import { u64Number } from "./u64Number";

import {
    MintInfo,
    MintLayout,
    u64

} from '@solana/spl-token';

declare global {
    export interface String {
        toPublicKey(): PublicKey;
    }
}

String.prototype.toPublicKey = function (): PublicKey {
    return new PublicKey(this.toString());
}

let defaultStreamInfo: StreamInfo = {
    id: undefined,
    initialized: false,
    memo: "",
    treasurerAddress: undefined,
    rateAmount: 0,
    rateIntervalInSeconds: 0,
    startUtc: undefined,
    rateCliffInSeconds: 0,
    cliffVestAmount: 0,
    cliffVestPercent: 0,
    beneficiaryAddress: undefined,
    associatedToken: undefined,
    escrowVestedAmount: 0,
    escrowUnvestedAmount: 0,
    treasuryAddress: undefined,
    escrowEstimatedDepletionUtc: undefined,
    totalDeposits: 0,
    totalWithdrawals: 0,
    isStreaming: false,
    isUpdatePending: false,
    transactionSignature: undefined,
    blockTime: 0
}

function parseStreamData(
    streamId: PublicKey,
    streamData: Buffer,
    friendly: boolean = true

): StreamInfo {

    let stream: StreamInfo = defaultStreamInfo;
    let decodedData = Layout.streamLayout.decode(streamData);
    const beneficiaryAssociatedToken = new PublicKey(decodedData.stream_associated_token);
    const associatedToken = beneficiaryAssociatedToken.toBase58() !== Constants.DEFAULT_PUBLICKEY
        ? beneficiaryAssociatedToken.toBase58()
        : (friendly ? beneficiaryAssociatedToken.toBase58() : beneficiaryAssociatedToken);

    let startDateUtc = new Date(decodedData.start_utc as string);
    let utcNow = new Date();
    let rateIntervalInSeconds = parseFloat(u64Number.fromBuffer(decodedData.rate_interval_in_seconds).toString());
    const rate = decodedData.rate_amount / rateIntervalInSeconds;
    const elapsedTime = (utcNow.getTime() - startDateUtc.getTime()) / 1000;

    let escrowVestedAmount = 0;

    if (utcNow.getTime() >= startDateUtc.getTime()) {
        escrowVestedAmount = rate * elapsedTime;

        if (escrowVestedAmount >= decodedData.total_deposits - decodedData.total_withdrawals) {
            escrowVestedAmount = decodedData.total_deposits - decodedData.total_withdrawals;
        }
    }

    let escrowEstimatedDepletionUtc = decodedData.escrow_estimated_depletion_utc;
    let escrowEstimatedDepletionDateUtc = new Date();

    escrowEstimatedDepletionDateUtc.setDate(escrowEstimatedDepletionUtc);

    let nameBuffer = Buffer
        .alloc(decodedData.stream_name.length, decodedData.stream_name)
        .filter(function (elem, index) {
            return elem !== 0;
        });

    const id = friendly !== undefined ? streamId.toBase58() : streamId;
    const treasurerAddress = new PublicKey(decodedData.treasurer_address);
    const beneficiaryAddress = new PublicKey(decodedData.beneficiary_address);
    const treasuryAddress = new PublicKey(decodedData.treasury_address);

    Object.assign(stream, { id: id }, {
        initialized: decodedData.initialized,
        memo: new TextDecoder().decode(nameBuffer),
        treasurerAddress: friendly !== undefined ? treasurerAddress.toBase58() : treasurerAddress,
        rateAmount: decodedData.rate_amount,
        rateIntervalInSeconds: rateIntervalInSeconds,
        startUtc: startDateUtc.toUTCString(),
        rateCliffInSeconds: parseFloat(u64Number.fromBuffer(decodedData.rate_cliff_in_seconds).toString()),
        cliffVestAmount: decodedData.cliff_vest_amount,
        cliffVestPercent: decodedData.cliff_vest_percent,
        beneficiaryAddress: friendly !== undefined ? beneficiaryAddress.toBase58() : beneficiaryAddress,
        associatedToken: associatedToken,
        escrowVestedAmount: escrowVestedAmount,
        escrowUnvestedAmount: decodedData.total_deposits - decodedData.total_withdrawals - escrowVestedAmount,
        treasuryAddress: friendly !== undefined ? treasuryAddress.toBase58() : treasuryAddress,
        escrowEstimatedDepletionUtc: escrowEstimatedDepletionDateUtc.toUTCString(),
        totalDeposits: decodedData.total_deposits,
        totalWithdrawals: decodedData.total_withdrawals,
        isStreaming: escrowVestedAmount < decodedData.total_deposits && decodedData.rate_amount > 0,
        isUpdatePending: false,
        transactionSignature: '',
        blockTime: 0
    });

    return stream;
}

export async function getStream(
    connection: Connection,
    id: PublicKey,
    commitment?: any,
    friendly: boolean = true

): Promise<StreamInfo> {

    let stream;
    let accountInfo = await connection.getAccountInfo(id, commitment);

    if (accountInfo?.data !== undefined && accountInfo?.data.length > 0) {

        let signatures = await connection.getConfirmedSignaturesForAddress2(id, {}, 'confirmed');

        if (signatures.length > 0) {

            stream = Object.assign({}, parseStreamData(
                id,
                accountInfo?.data,
                friendly
            ));

            stream.transactionSignature = signatures[0].signature;
            stream.blockTime = signatures[0].blockTime as number;
        }
    }

    return stream as StreamInfo;
}

export async function listStreams(
    connection: Connection,
    programId: PublicKey,
    treasurer?: PublicKey | undefined,
    beneficiary?: PublicKey | undefined,
    commitment?: any,
    friendly: boolean = true

): Promise<StreamInfo[]> {

    let streams: StreamInfo[] = [];
    const accounts = await connection.getProgramAccounts(programId, commitment);

    if (accounts === null || !accounts.length) {
        return streams;
    }

    for (let item of accounts) {
        if (item.account.data !== undefined && item.account.data.length === Layout.streamLayout.span) {

            let included = false;
            let info = Object.assign({}, parseStreamData(
                item.pubkey,
                item.account.data,
                friendly
            ));

            if ((treasurer && treasurer.toBase58() === info.treasurerAddress) || !treasurer) {
                included = true;
            } else if ((beneficiary && beneficiary.toBase58() === info.beneficiaryAddress) || !beneficiary) {
                included = true;
            }

            if (included && (info.startUtc as Date) !== undefined) {

                let startDateUtc = new Date(info.startUtc as string);
                let threeDaysAgo = new Date();
                threeDaysAgo.setDate(threeDaysAgo.getDate() - 3);

                if (startDateUtc.getTime() > threeDaysAgo.getTime()) {

                    let signatures = await connection.getConfirmedSignaturesForAddress2(
                        (friendly ? (info.id as string).toPublicKey() : (info.id as PublicKey)),
                        {}, commitment
                    );

                    if (signatures.length > 0) {
                        info.blockTime = signatures[0].blockTime as number;
                        info.transactionSignature = signatures[0].signature
                    }
                }
            }

            if (included) {
                streams.push(info);
            }

        }
    }

    let orderedStreams = streams.sort((a, b) => (b.blockTime - a.blockTime));

    return orderedStreams;
}

export function getSeedBuffer(
    from: PublicKey,
    programId: PublicKey

): (Buffer | Uint8Array)[] {

    return [
        from.toBytes(),
        programId.toBytes(),
        new TextEncoder().encode('MoneyStreamingProgram')
    ];
}

export async function findMSPAddress(
    from: PublicKey,
    programId: PublicKey

): Promise<[PublicKey, number]> {

    const seedBuffer = getSeedBuffer(from, programId);

    return (
        await PublicKey.findProgramAddress(
            seedBuffer,
            programId
        )
    );
}

export async function findMSPStreamAddress(
    treasurer: PublicKey,
    programId: PublicKey

): Promise<[PublicKey, number]> {

    const seedBuffer = getSeedBuffer(treasurer, programId);

    return (
        await PublicKey.findProgramAddress(
            seedBuffer,
            programId
        )
    );
}

export async function findMSPTreasuryAddress(
    stream: PublicKey,
    programId: PublicKey

): Promise<[PublicKey, number]> {

    const seedBuffer = getSeedBuffer(stream, programId);

    return (
        await PublicKey.findProgramAddress(
            seedBuffer,
            programId
        )
    );
}

export async function findATokenAddress(
    walletAddress: PublicKey,
    tokenMintAddress: PublicKey

): Promise<PublicKey> {

    return (
        await PublicKey.findProgramAddress(
            [
                walletAddress.toBuffer(),
                Constants.TOKEN_PROGRAM_ADDRESS.toPublicKey().toBuffer(),
                tokenMintAddress.toBuffer(),
            ],
            Constants.ATOKEN_PROGRAM_ADDRESS.toPublicKey()
        )
    )[0];
}

export function toNative(amount: number) {
    return new BN(amount * 10 ** Constants.DECIMALS);
}

export function fromNative(amount: BN) {
    return amount.toNumber() / 10 ** Constants.DECIMALS;
}

export const getMintAccount = async (
    connection: Connection,
    pubKey: PublicKey | string

): Promise<MintInfo> => {

    const address = typeof pubKey === 'string' ? new PublicKey(pubKey) : pubKey;
    const info = await connection.getAccountInfo(address);

    if (info === null) {
        throw new Error('Failed to find mint account');
    }

    return deserializeMint(info.data);
};

export const deserializeMint = (data: Buffer): MintInfo => {
    if (data.length !== MintLayout.span) {
        throw new Error('Not a valid Mint');
    }

    const mintInfo = MintLayout.decode(data);

    if (mintInfo.mintAuthorityOption === 0) {
        mintInfo.mintAuthority = null;
    } else {
        mintInfo.mintAuthority = new PublicKey(mintInfo.mintAuthority);
    }

    mintInfo.supply = u64.fromBuffer(mintInfo.supply);
    mintInfo.isInitialized = mintInfo.isInitialized !== 0;

    if (mintInfo.freezeAuthorityOption === 0) {
        mintInfo.freezeAuthority = null;
    } else {
        mintInfo.freezeAuthority = new PublicKey(mintInfo.freezeAuthority);
    }

    return mintInfo as MintInfo;
};

export function convertLocalDateToUTCIgnoringTimezone(date: Date) {
    const timestamp = Date.UTC(
        date.getFullYear(),
        date.getMonth(),
        date.getDate(),
        date.getHours(),
        date.getMinutes(),
        date.getSeconds(),
        date.getMilliseconds(),
    );

    return new Date(timestamp);
}
