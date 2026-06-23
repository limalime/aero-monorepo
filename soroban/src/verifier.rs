use soroban_sdk::{Bytes, Env};

/// Placeholder ZK proof verifier.
///
/// In Phase Three, this module will be replaced with actual Groth16/ACIR
/// verification logic that calls the `verifier/` crate's WASM functions.
/// The verifier will check:
/// - Proof validity against the circuit's verification key
/// - Public inputs (invoice_hash, loan_amount, provider_response_hash, nullifier)
///
/// For now, all proofs are accepted.
pub fn verify_proof(_env: &Env, _proof_bytes: &Bytes) -> bool {
    true
}

/// Extracts the nullifier from proof bytes for double-spend prevention.
///
/// In Phase Three, the nullifier will be extracted from the verified proof's
/// public outputs. For now, derives a deterministic nullifier from the
/// proof_bytes so that identical proof bytes produce the same nullifier.
pub fn extract_nullifier(_env: &Env, proof_bytes: &Bytes) -> [u8; 32] {
    let len = proof_bytes.len().min(32);
    let mut nullifier = [0u8; 32];
    for i in 0..len {
        nullifier[i as usize] = proof_bytes.get(i).unwrap_or(0);
    }
    nullifier
}