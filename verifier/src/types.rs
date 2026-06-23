use soroban_sdk::{contracterror, contracttype, BytesN};

// --- Public Inputs from Proof ---

/// Public inputs extracted from a verified Noir UltraHonk proof.
/// These values are consumed by the Aero soroban contract to issue loans.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PublicInputs {
    /// Nullifier preventing double-spend (Pedersen hash of invoice_id + seed)
    pub nullifier: BytesN<32>,
    /// Pedersen commitment to the invoice data (domain-separated with v1 tag)
    pub invoice_hash: BytesN<32>,
    /// Computed loan amount in stroops or USDC cents
    pub loan_amount: i128,
    /// Loan-to-value ratio in basis points from circuit output
    pub ltv_bps: u32,
    /// Interest rate in basis points per annum from circuit output
    pub interest_bps: u32,
    /// Pedersen hash binding provider identity to TLS response commitment
    pub provider_response_hash: BytesN<32>,
}

// --- Verification Key ---

/// The Noir UltraHonk Verification Key, stored as a hash reference.
/// The full VK bytes are stored off-chain; only the hash is kept on-chain
/// to identify which VK was used during verification.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VerificationKey {
    /// Blake2s hash of the full VK bytes
    pub vk_hash: BytesN<32>,
    /// The curve identifier (always BN254 for Stellar)
    pub curve: u32,
    /// Number of public inputs expected by the circuit
    pub num_public_inputs: u32,
    /// Proof system identifier (0 = UltraHonk)
    pub proof_system: u32,
}

// --- Proof ---

/// Raw proof bytes submitted for on-chain verification.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofData {
    /// The raw proof bytes (compressed UltraHonk proof)
    pub proof_bytes: soroban_sdk::Bytes,
    /// Public inputs serialized in Noir's format
    pub public_inputs_bytes: soroban_sdk::Bytes,
}

// --- Storage Keys ---

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract administrator
    Admin,
    /// The active Verification Key
    VerificationKey,
    /// Whether the contract has been initialized
    Initialized,
    /// Registry of verified nullifiers (double-spend prevention at verifier level)
    NullifierVerified(BytesN<32>),
}

// --- Contract Errors ---

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ProofVerificationFailed = 4,
    InvalidProofFormat = 5,
    InvalidVkHash = 6,
    VkNotSet = 7,
    NullifierAlreadyVerified = 8,
    PublicInputMismatch = 9,
}

// --- Protocol Constants ---

/// Curve identifier for BN254 (Stellar Protocol 25/26).
pub const CURVE_BN254: u32 = 1;
/// Proof system identifier for UltraHonk.
pub const PROOF_SYSTEM_ULTRAHONK: u32 = 0;
/// Expected number of public inputs from the Aero circuit.
pub const AERO_NUM_PUBLIC_INPUTS: u32 = 8;
/// Size of a single proof field in bytes (BN254 scalar field).
pub const FIELD_SIZE: u32 = 32;