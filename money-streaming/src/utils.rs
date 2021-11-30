
use std::{ string::String, convert::TryInto };
use solana_program::pubkey::Pubkey;
use crate::error::{ StreamError, TreasuryError };

pub fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), StreamError> {
    if input.len() >= 32 {
        let (key, rest) = input.split_at(32);
        let pk = Pubkey::new(key);

        Ok((pk, rest))
    } else {
        Err(StreamError::InvalidArgument.into())
    }
}

pub fn unpack_string(input: &[u8]) -> Result<(String, &[u8]), StreamError> {
    if input.len() >= 32 {
        let (bytes, rest) = input.split_at(32);
        Ok((String::from_utf8_lossy(bytes).to_string(), rest))
    } else {
        Err(StreamError::InvalidArgument.into())
    }
}

pub fn unpack_u64(input: &[u8]) -> Result<u64, StreamError> {
    let amount = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(StreamError::InvalidStreamInstruction)?;

    Ok(amount)
}

pub fn unpack_f64(input: &[u8]) -> Result<f64, StreamError> {
    let amount = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(f64::from_le_bytes)
        .ok_or(StreamError::InvalidStreamInstruction)?;

    Ok(amount)
}