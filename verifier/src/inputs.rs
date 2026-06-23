use soroban_sdk::{Bytes, BytesN, Env};

use crate::types::{Error, PublicInputs, AERO_NUM_PUBLIC_INPUTS, FIELD_SIZE};

/// Extracts public inputs from a verified Noir proof's public output bytes.
///
/// Noir encodes public inputs as a flat sequence of BN254 scalar field elements
/// (each 32 bytes, little-endian). The Aero circuit produces 8 public fields:
///
/// | Index | Field                 | Type      |
/// |-------|----------------------|-----------|
/// | 0     | invoice_hash         | Field     |
/// | 1     | loan_amount          | u64       |
/// | 2     | provider_response_hash | Field   |
/// | 3     | nullifier            | Field     |
/// | 4     | ltv_bps              | u32       |
/// | 5     | interest_bps          | u32       |
/// | 6     | reserved             | Field     |
/// | 7     | reserved             | Field     |
///
/// Returns a fully parsed `PublicInputs` struct ready for the soroban contract.
pub fn extract_public_inputs(
    env: &Env,
    public_inputs_bytes: &Bytes,
) -> Result<PublicInputs, Error> {
    let expected_len = AERO_NUM_PUBLIC_INPUTS * FIELD_SIZE;

    if public_inputs_bytes.len() != expected_len {
        return Err(Error::InvalidProofFormat);
    }

    // Parse each field as 32-byte little-endian value
    let invoice_hash = read_field(env, public_inputs_bytes, 0);
    let loan_amount_raw = read_field(env, public_inputs_bytes, 1);
    let provider_response_hash = read_field(env, public_inputs_bytes, 2);
    let nullifier = read_field(env, public_inputs_bytes, 3);
    let ltv_bps_raw = read_field(env, public_inputs_bytes, 4);
    let interest_bps_raw = read_field(env, public_inputs_bytes, 5);
    // Fields 6 and 7 are reserved, skipped

    // Extract loan_amount from the field value (stored as u64 in the first 8 bytes)
    let loan_amount = extract_u64_from_field(&loan_amount_raw);

    // Extract LTV and interest from field values (stored as u32)
    let ltv_bps = extract_u32_from_field(&ltv_bps_raw);
    let interest_bps = extract_u32_from_field(&interest_bps_raw);

    Ok(PublicInputs {
        nullifier,
        invoice_hash,
        loan_amount,
        ltv_bps,
        interest_bps,
        provider_response_hash,
    })
}

/// Reads a 32-byte field from the serialized public inputs at the given index.
fn read_field(env: &Env, bytes: &Bytes, index: u32) -> BytesN<32> {
    let start = index * FIELD_SIZE;
    let mut arr = [0u8; 32];
    for i in 0..32u32 {
        arr[i as usize] = bytes.get(start + i).unwrap_or(0);
    }
    BytesN::from_array(env, &arr)
}

/// Extracts a u32 from the first 4 bytes of a 32-byte field (little-endian).
fn extract_u32_from_field(field: &BytesN<32>) -> u32 {
    let mut result: u32 = 0;
    for i in 0..4 {
        let byte = field.get(i).unwrap_or(0) as u32;
        result |= byte << (i * 8);
    }
    result
}

/// Extracts a u64 from the first 8 bytes of a 32-byte field (little-endian),
/// then casts to i128 for the loan_amount field.
///
/// The Noir circuit outputs loan_amount as a u64. Reading exactly 8 bytes
/// prevents adversarial data in the upper bytes from inflating the value.
fn extract_u64_from_field(field: &BytesN<32>) -> i128 {
    let mut result: u64 = 0;
    for i in 0..8 {
        let byte = field.get(i).unwrap_or(0) as u64;
        result |= byte << (i * 8);
    }
    result as i128
}

/// Validates that the extracted public inputs are internally consistent.
///
/// Checks that loan_amount, ltv_bps, and interest_bps are within valid ranges.
pub fn validate_public_inputs(inputs: &PublicInputs) -> Result<(), Error> {
    if inputs.loan_amount <= 0 {
        return Err(Error::PublicInputMismatch);
    }
    if inputs.ltv_bps == 0 || inputs.ltv_bps > 10000 {
        return Err(Error::PublicInputMismatch);
    }
    if inputs.interest_bps == 0 || inputs.interest_bps > 10000 {
        return Err(Error::PublicInputMismatch);
    }
    Ok(())
}