#![no_std]

pub mod honk;
pub mod inputs;
pub mod keys;
pub mod types;

use soroban_sdk::{Bytes, BytesN, Env};

use crate::types::{DataKey, Error, ProofData, PublicInputs};

/// Verifies a combined proof + public inputs and extracts the public outputs.
///
/// This is a library-level convenience function for use by the Aero soroban
/// contract. The `proof_bytes` contain the full Noir proof followed by the
/// 256-byte serialized public inputs (8 fields x 32 bytes). The verifier
/// splits the combined bytes internally and performs:
/// 1. Public input extraction and validation
/// 2. Nullifier double-spend check
///
/// `vk_hash_ref` is the 32-byte VK identifier. In mock mode, VK validation
/// is skipped; the hash is accepted for forward compatibility with Protocol
/// 25/26 BN254 host functions that will perform real VK binding.
pub fn verify(
    env: &Env,
    proof_bytes: &Bytes,
    vk_hash_ref: &BytesN<32>,
) -> Result<PublicInputs, Error> {
    let total = proof_bytes.len();
    if total < 256 {
        return Err(Error::InvalidProofFormat);
    }

    // Extract last 256 bytes as public inputs
    let pub_start = total - 256;
    let mut pub_arr = [0u8; 256];
    for i in 0..256u32 {
        pub_arr[i as usize] = proof_bytes.get(pub_start + i).unwrap_or(0);
    }
    let public_inputs_part = Bytes::from_slice(env, &pub_arr);

    // Convert vk_hash_ref to Bytes for forward compatibility
    let mut vk_arr = [0u8; 32];
    for i in 0..32u32 {
        vk_arr[i as usize] = vk_hash_ref.get(i).unwrap_or(0);
    }
    let _vk_bytes = Bytes::from_slice(env, &vk_arr);

    // Proof part is not used in mock mode
    let proof = ProofData {
        proof_bytes: Bytes::from_slice(env, &[]),
        public_inputs_bytes: public_inputs_part,
    };

    // Extract and validate public inputs directly (skip VK validation in mock mode)
    let public_inputs = crate::inputs::extract_public_inputs(env, &proof.public_inputs_bytes)?;
    crate::inputs::validate_public_inputs(&public_inputs)?;

    // Check nullifier hasn't been seen before
    let nullifier_key = DataKey::NullifierVerified(public_inputs.nullifier.clone());
    if env.storage().persistent().has(&nullifier_key) {
        return Err(Error::NullifierAlreadyVerified);
    }
    env.storage().persistent().set(&nullifier_key, &true);
    env.storage().persistent().extend_ttl(&nullifier_key, 535680, 535680);

    Ok(public_inputs)
}