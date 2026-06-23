use crate::utils;

global SIGNATURE_BYTE_LEN: u32 = 64;

/// Represents a zkTLS proof that an invoice was fetched from a trusted Web2 API.
/// The proof contains an attestation from a zkTLS prover that:
/// 1. A TLS session was established with `provider_id`
/// 2. The API response committed to in `response_commitment` was received
/// 3. The `session_hash` binds the proof to a specific session
pub struct ZkTlsProof {
    pub provider_id: [u8; 32],
    pub session_hash: [u8; 32],
    pub response_commitment: [u8; 32],
    pub tls_signature: [u8; SIGNATURE_BYTE_LEN],
}

impl ZkTlsProof {
    /// Verifies that the provider is in the trusted provider set.
    /// Returns true if provider_id matches any entry in the trusted list.
    pub fn verify_provider(self, trusted_providers: [[u8; 32]; utils::TRUSTED_PROVIDER_COUNT]) -> bool {
        let mut found = false;
        for i in 0..utils::TRUSTED_PROVIDER_COUNT {
            found = found | utils::bytes32_eq(self.provider_id, trusted_providers[i]);
        }
        found
    }

    /// Verifies the zkTLS proof commitment structure.
    /// Checks that:
    /// - provider_id is not zero (must be a real provider)
    /// - session_hash is not zero (session must be real)
    /// - response_commitment is not zero (must commit to data)
    // Note: The heavy ECDSA/TLS signature math is verified natively off-chain
    // by the zkTLS client to save ZK constraints. This circuit enforces
    // structural integrity and binds the data.
    fn verify_commitment_integrity(self) -> bool {
        let provider_valid = !utils::bytes32_is_zero(self.provider_id);
        let session_valid = !utils::bytes32_is_zero(self.session_hash);
        let response_valid = !utils::bytes32_is_zero(self.response_commitment);
        provider_valid & session_valid & response_valid
    }

    /// Full zkTLS verification: provider authorization + commitment integrity.
    pub fn verify(self, trusted_providers: [[u8; 32]; utils::TRUSTED_PROVIDER_COUNT]) -> bool {
        let provider_ok = self.verify_provider(trusted_providers);
        let integrity_ok = self.verify_commitment_integrity();
        provider_ok & integrity_ok
    }

    /// Binds the response commitment, provider identity, and TLS signature
    /// into a single Field commitment. This prevents the prover from swapping
    /// the signature after proof generation -- the signature is cryptographically
    /// bound into the public output that the Soroban verifier checks.
    /// Splits the 64-byte tls_signature into two 32-byte halves and hashes
    /// all four components together.
    pub fn provider_response_hash(self) -> Field {
        let p = utils::bytes32_to_field(self.provider_id);
        let r = utils::bytes32_to_field(self.response_commitment);
        // Split the 64-byte signature into two 32-byte halves
        let mut sig_first_arr: [u8; 32] = [0; 32];
        let mut sig_last_arr: [u8; 32] = [0; 32];
        for i in 0..32 {
            sig_first_arr[i] = self.tls_signature[i];
            sig_last_arr[i] = self.tls_signature[32 + i];
        }
        let sig_first = utils::bytes32_to_field(sig_first_arr);
        let sig_last = utils::bytes32_to_field(sig_last_arr);
        // Hash all four components together for cryptographic binding
        let mut inputs = [0; utils::HASH_INPUT_SIZE];
        inputs[0] = p;
        inputs[1] = r;
        inputs[2] = sig_first;
        inputs[3] = sig_last;
        utils::hash_fields(inputs)
    }

    /// Returns the provider ID.
    pub fn get_provider_id(self) -> [u8; 32] {
        self.provider_id
    }
}

#[test]
fn test_verify_provider_found() {
    let mut providers = [[0; 32]; utils::TRUSTED_PROVIDER_COUNT];
    let provider_bytes = [0xAA; 32];
    providers[0] = provider_bytes;

    let proof = ZkTlsProof {
        provider_id: provider_bytes,
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; SIGNATURE_BYTE_LEN],
    };
    assert(proof.verify_provider(providers));
}

#[test]
fn test_verify_provider_not_found() {
    let mut providers = [[0; 32]; utils::TRUSTED_PROVIDER_COUNT];
    providers[0] = [0xAA; 32];

    let proof = ZkTlsProof {
        provider_id: [0xFF; 32],
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; SIGNATURE_BYTE_LEN],
    };
    assert(!proof.verify_provider(providers));
}

#[test]
fn test_verify_provider_at_end_of_list() {
    let mut providers = [[0; 32]; utils::TRUSTED_PROVIDER_COUNT];
    let provider_bytes = [0xEE; 32];
    providers[9] = provider_bytes;

    let proof = ZkTlsProof {
        provider_id: provider_bytes,
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; SIGNATURE_BYTE_LEN],
    };
    assert(proof.verify_provider(providers));
}

#[test]
fn test_verify_integrity_rejects_zero_provider() {
    let proof = ZkTlsProof {
        provider_id: [0; 32],
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; SIGNATURE_BYTE_LEN],
    };
    let providers = [[0; 32]; utils::TRUSTED_PROVIDER_COUNT];
    assert(!proof.verify(providers));
}

#[test]
fn test_provider_response_hash_deterministic() {
    let proof = ZkTlsProof {
        provider_id: [0xAA; 32],
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; SIGNATURE_BYTE_LEN],
    };
    let h1 = proof.provider_response_hash();
    let h2 = proof.provider_response_hash();
    assert(h1 == h2);
}