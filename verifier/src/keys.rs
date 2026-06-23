use soroban_sdk::{BytesN, Env};

use crate::types::{DataKey, Error, VerificationKey, CURVE_BN254, PROOF_SYSTEM_ULTRAHONK};

/// Stores the Verification Key hash in persistent contract storage.
///
/// The full VK bytes are too large for on-chain storage (~16KB for UltraHonk).
/// Only the Blake2s hash is stored on-chain; the verifier contract uses this
/// hash to confirm the VK was provided by the caller during proof verification.
pub fn set_vk(env: &Env, vk_hash: BytesN<32>, num_public_inputs: u32) -> Result<(), Error> {
    let vk = VerificationKey {
        vk_hash,
        curve: CURVE_BN254,
        num_public_inputs,
        proof_system: PROOF_SYSTEM_ULTRAHONK,
    };
    let key = DataKey::VerificationKey;
    env.storage().persistent().set(&key, &vk);
    // Extend TTL to maximum so the VK never expires
    env.storage().persistent().extend_ttl(&key, 535680, 535680);
    Ok(())
}

/// Retrieves the stored Verification Key from persistent storage.
pub fn get_vk(env: &Env) -> Result<VerificationKey, Error> {
    env.storage()
        .persistent()
        .get::<DataKey, VerificationKey>(&DataKey::VerificationKey)
        .ok_or(Error::VkNotSet)
}

/// Validates that the provided VK hash matches the stored VK.
pub fn validate_vk(env: &Env, expected_vk_hash: &BytesN<32>) -> Result<(), Error> {
    let stored = get_vk(env)?;
    if stored.vk_hash != *expected_vk_hash {
        return Err(Error::InvalidVkHash);
    }
    Ok(())
}

/// Computes a Blake2s hash of VK bytes for on-chain storage reference.
///
/// In Phase Three, this is a placeholder. In production, the VK hash
/// is computed off-chain and passed during initialization.
// SECURITY WARNING: This is NOT a real hash. In mock mode, the caller
// can trivially satisfy VK validation by passing any 32-byte vk_bytes
// whose first 32 bytes match the stored vk_hash. VK binding has NO
// cryptographic security until env.crypto().blake2s() or a real ZK hash
// is integrated in the production release.
pub fn compute_vk_hash(env: &Env, vk_bytes: &soroban_sdk::Bytes) -> BytesN<32> {
    // TODO: Use env.crypto().blake2s() when available in Protocol 25/26
    // For now, returns the first 32 bytes of the VK as a mock hash.
    // In production, use a real Blake2s hash of the full VK bytes.
    let len = vk_bytes.len().min(32);
    let mut arr = [0u8; 32];
    for i in 0..len {
        arr[i as usize] = vk_bytes.get(i).unwrap_or(0);
    }
    BytesN::from_array(env, &arr)
}