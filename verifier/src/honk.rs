use soroban_sdk::{Bytes, Env};

use crate::inputs;
use crate::keys;
use crate::types::{Error, ProofData, PublicInputs};

/// Verifies an UltraHonk proof against the stored Verification Key.
///
/// This function performs the core cryptographic verification of a Noir
/// UltraHonk proof using Stellar's Protocol 25/26 BN254 host functions.
///
/// # Protocol 25/26 BN254 Host Functions Used:
/// - `env.crypto().bn254_msm()` -- Multi-scalar multiplication on G1/G2
/// - `env.crypto().bn254_pairing_check()` -- Pairing check for Groth16/Plonk
/// - `env.crypto().bn254_hash_to_field()` -- Hash-to-field for Fiat-Shamir
///
/// # Verification Steps:
/// 1. Validate VK hash matches stored VK
/// 2. Deserialize proof into constituent parts (commitments, openings, IPA proof)
/// 3. Compute challenges via Fiat-Shamir (hash transcript)
/// 4. Check pairing equation using BN254 host functions
///
/// For Phase Three, this is a structured placeholder. The full UltraHonk
/// verification with native BN254 host functions will be finalized when
/// Protocol 25/26 reaches mainnet and soroban-sdk exposes the crypto APIs.
pub fn verify_proof(
    env: &Env,
    proof: &ProofData,
    vk_bytes: &Bytes,
) -> Result<PublicInputs, Error> {
    // Step 1: Validate the VK hash
    let vk_hash = keys::compute_vk_hash(env, vk_bytes);
    keys::validate_vk(env, &vk_hash)?;

    // Step 2: Verify the proof cryptographically
    //
    // TODO: Implement actual UltraHonk verification using BN254 host functions:
    //
    //   // Deserialize proof
    //   let honk_proof = deserialize_ultrahonk_proof(env, &proof.proof_bytes)?;
    //
    //   // Compute challenges via Fiat-Shamir
    //   let challenges = compute_challenges(env, &honk_proof, &vk_bytes)?;
    //
    //   // Multi-scalar multiplication check
    //   let msm_result = env.crypto().bn254_msm(
    //       &honk_proof.commitments,
    //       &challenges.scalars,
    //   )?;
    //
    //   // Pairing check
    //   let valid = env.crypto().bn254_pairing_check(
    //       &msm_result,
    //       &honk_proof.openings,
    //   )?;
    //
    //   if !valid { return Err(Error::ProofVerificationFailed); }
    //
    // For now, accept all proofs (mock mode for development)

    // Step 3: Extract and validate public inputs
    let public_inputs = inputs::extract_public_inputs(env, &proof.public_inputs_bytes)?;
    inputs::validate_public_inputs(&public_inputs)?;

    Ok(public_inputs)
}

/// Verifies a proof and checks the nullifier has not been used before.
///
/// This is the main entry point called by the Aero soroban contract.
/// It wraps `verify_proof` with an additional nullifier check for
/// double-spend prevention at the verifier level.
pub fn verify_and_check_nullifier(
    env: &Env,
    proof: &ProofData,
    vk_bytes: &Bytes,
) -> Result<PublicInputs, Error> {
    let public_inputs = verify_proof(env, proof, vk_bytes)?;

    // Check nullifier hasn't been seen before
    let nullifier_key =
        crate::types::DataKey::NullifierVerified(public_inputs.nullifier.clone());
    if env.storage().persistent().has(&nullifier_key) {
        return Err(Error::NullifierAlreadyVerified);
    }
    env.storage().persistent().set(&nullifier_key, &true);
    // Extend TTL to maximum so the nullifier never expires
    env.storage().persistent().extend_ttl(&nullifier_key, 535680, 535680);

    Ok(public_inputs)
}