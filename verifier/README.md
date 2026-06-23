# Aero Verifier

Cryptographic bridge connecting Noir ZK circuits to the Aero Soroban smart contracts. This contract verifies Noir UltraHonk proofs using Stellar's Protocol 25/26 BN254 host functions and extracts public inputs for on-chain loan issuance.

## Architecture

```
verifier/
├── Cargo.toml          # Package manifest (cdylib + rlib)
├── src/
│   ├── lib.rs          # VerifierContract: init, verify, verify_without_nullifier, admin
│   ├── honk.rs         # UltraHonk proof verification + nullifier check
│   ├── inputs.rs       # Public input extraction from Noir's serialized format
│   ├── keys.rs         # Verification Key storage and hash validation
│   └── types.rs        # PublicInputs, VerificationKey, ProofData, errors, constants
├── tests/
│   └── test.rs         # 7 tests: VK management, proof verification, nullifier, format validation
└── README.md
```

## Verification Flow

```
Off-chain:                          On-chain:
                                  ┌──────────────────────┐
Noir circuit ──► nargo prove      │  VerifierContract     │
                    │              │                      │
                    ▼              │  init(vk_hash)       │
              proof + public       │                      │
              inputs bytes         │  verify(proof,       │
                    │              │    public_inputs,    │
                    └──────────────┤    vk_bytes)         │
                                   │    ┌─────────────────┤
                                   │    │ 1. validate VK  │
                                   │    │ 2. verify proof │
                                   │    │ 3. extract PI   │
                                   │    │ 4. check nullif.│
                                   │    └─────────────────┤
                                   │         │            │
                                   │         ▼            │
                                   │   PublicInputs {     │
                                   │     nullifier,       │
                                   │     invoice_hash,    │
                                   │     loan_amount,     │
                                   │     ltv_bps,         │
                                   │     interest_bps,    │
                                   │     provider_hash    │
                                   │   }                  │
                                   └──────────┬───────────┘
                                              │
                                              ▼
                                   Aero Soroban Contract
                                   (issues loan)
```

## Public Input Format

Noir encodes public inputs as a flat sequence of 32-byte BN254 scalar field elements (little-endian). The Aero circuit produces 8 fields:

| Index | Field | Type | Description |
|-------|-------|------|-------------|
| 0 | `invoice_hash` | Field (32 bytes) | Pedersen commitment to invoice data |
| 1 | `loan_amount` | u64 (in first 8 bytes) | Loan amount in stroops/cents |
| 2 | `provider_response_hash` | Field (32 bytes) | Provider identity + TLS signature binding |
| 3 | `nullifier` | Field (32 bytes) | Double-spend prevention identifier |
| 4 | `ltv_bps` | u32 (in first 4 bytes) | Loan-to-value ratio in basis points |
| 5 | `interest_bps` | u32 (in first 4 bytes) | Interest rate in basis points |
| 6 | reserved | Field (32 bytes) | Reserved for future use |
| 7 | reserved | Field (32 bytes) | Reserved for future use |

Total: 8 x 32 = 256 bytes.

## Contract Interface

| Function | Description |
|----------|-------------|
| `init(admin, vk_hash, num_public_inputs)` | Initialize with admin and Verification Key hash |
| `verify(proof_bytes, public_inputs_bytes, vk_bytes)` | Verify proof, extract public inputs, check nullifier |
| `verify_without_nullifier(proof_bytes, public_inputs_bytes, vk_bytes)` | Verify without nullifier check (pre-flight validation) |
| `get_vk()` | Query stored Verification Key metadata |
| `get_admin()` | Query contract admin |

## Extracting the Verification Key from Noir

The Verification Key (VK) is produced when compiling the Noir circuit. Steps:

```bash
# 1. Compile the Noir circuit (Phase One)
cd circuits/
nargo compile

# 2. The compiled circuit is in circuits/target/
#    The VK is embedded in the compiled artifact.

# 3. For UltraHonk, extract the VK bytes:
#    (exact command depends on Noir version and backend)
nargo check --write-vk

# 4. Compute Blake2s hash of the VK (for on-chain storage):
#    Use the Noir std lib or an external tool:
#    echo "VK_HASH=$(blake2s vk.bin)" 
```

Once the VK hash is obtained, initialize the verifier:

```bash
stellar contract invoke \
  --id <VERIFIER_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- init \
  --admin alice \
  --vk_hash <VK_HASH> \
  --num_public_inputs 8
```

## Testing

```bash
cargo test
```

7 tests covering:
- VK initialization and retrieval
- Double initialization prevention
- Public input extraction with field-by-field verification
- Nullifier double-spend prevention
- Invalid proof format rejection
- `verify_without_nullifier` (repeatable pre-flight check)
- Not-initialized guard

## Protocol 25/26 BN254 Integration (Phase Three)

When Stellar Protocol 25/26 reaches mainnet and `soroban-sdk` exposes the BN254 host functions, the following changes are needed:

### In `honk.rs` (`verify_proof`):

```rust
// Replace the mock acceptance with actual UltraHonk verification:

// 1. Deserialize the UltraHonk proof
let proof = deserialize_ultrahonk(&proof.proof_bytes)?;

// 2. Compute Fiat-Shamir challenges
let challenges = compute_challenges(&proof, &public_inputs_bytes)?;

// 3. Multi-scalar multiplication on G1
//    env.crypto().bn254_g1_msm(points, scalars)
let msm_result = env.crypto().bn254_g1_msm(
    &proof.commitments,
    &challenges.scalars,
)?;

// 4. Pairing check (Groth16-style)
let valid = env.crypto().bn254_pairing_check(
    &msm_result,
    &proof.openings,
)?;

if !valid {
    return Err(Error::ProofVerificationFailed);
}
```

### In `keys.rs` (`compute_vk_hash`):

```rust
// Use actual Blake2s host function
pub fn compute_vk_hash(env: &Env, vk_bytes: &Bytes) -> BytesN<32> {
    env.crypto().blake2s(vk_bytes)
}
```

## Security Model

- **VK binding**: The contract stores only the VK hash in persistent storage (max TTL); full VK is provided per-call and validated against the stored hash
- **Nullifier registry**: Each verified nullifier is stored in persistent storage with max TTL extension to prevent TTL-expiry replay attacks
- **VK persistence**: The Verification Key is stored in `persistent` storage (not `instance`) with TTL extended to maximum, preventing contract breakage on TTL expiry
- **Input validation**: Public inputs are validated for range and consistency before being returned
- **Admin-gated init**: Only the admin can set the VK hash, preventing unauthorized VK swaps
- **Mock VK warning**: The current `compute_vk_hash` is a plain byte slice (not a real hash) — a documented security limitation to be replaced with `env.crypto().blake2s()` in production