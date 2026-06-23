# Aero Circuits

Zero-Knowledge circuits for the Aero protocol. Built with [Noir](https://noir-lang.org/) 1.0.0-beta.22, targeting the BN254 curve with Pedersen hashing for Stellar Soroban verification. Pedersen is used because Poseidon is not yet available in this Noir version; when Poseidon lands (CAP-0075), the migration is a one-line change in `src/utils.nr`.

## Architecture

```
circuits/
├── src/
│   ├── main.nr              # Crate root: mod declarations + main circuit entry point
│   ├── utils.nr             # Constants, Pedersen hash helpers, validation primitives
│   ├── invoice.nr           # InvoiceData struct, field validation, commitment (domain-separated)
│   ├── zktls.nr             # ZkTlsProof struct, provider verification, signature binding
│   ├── lending.nr           # LendingParams struct, LTV enforcement, loan computation
│   ├── test_utils.nr        # Unit tests for utility functions (14 tests)
│   └── test_integration.nr  # End-to-end circuit flow tests (7 tests)
├── tests/
│   └── README.md            # Test documentation (tests run via nargo test)
├── Nargo.toml               # Project manifest
├── Prover.toml              # Example prover inputs
└── README.md
```

## Circuit Flow

1. **Input**: Private invoice data + zkTLS proof, public lending parameters + trusted provider list
2. **Invoice validation**: Amount range, currency, chronological dates, non-expired, age limit
3. **Lending enforcement**: LTV ratio computation, min/max amount checks
4. **zkTLS verification**: Provider is in trusted set, commitment integrity check
   - Heavy ECDSA/TLS signature math is verified off-chain by the zkTLS client
   - The circuit enforces structural integrity and binds the TLS signature to the public output
5. **Output**: Invoice hash, loan amount, provider-response-signature hash, nullifier (double-spend guard)

## Quick Start

```bash
# Check Noir installation
nargo --version

# Type-check the circuits
nargo check

# Run all tests (41 tests)
nargo test

# Generate a proof with example inputs
nargo execute
```

## Tests

```bash
nargo test
```

Test locations and coverage:
- `src/test_utils.nr` — byte conversion, hashing, loan computation, validation helpers
- `src/invoice.nr` — inline `#[test]`: validation, commitment determinism, collision resistance
- `src/zktls.nr` — inline `#[test]`: provider lookup, commitment integrity, signature binding
- `src/lending.nr` — inline `#[test]`: param validation, LTV enforcement, loan computation
- `src/main.nr` — inline `#[test]`: happy path, nullifier determinism, nullifier uniqueness
- `src/test_integration.nr` — cross-module: multi-currency, boundary values, multiple providers, near-expiry

## Circuit Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Curve | BN254 | Stellar Soroban compatible |
| Hash | Pedersen | ZK-friendly; Poseidon migration when available |
| Max invoice | $1,000,000 | Global protocol cap on invoice face value |
| Min invoice | $100 | Global protocol minimum; lenders may set higher |
| Max LTV | 100% | Configurable per loan via LendingParams |
| Max age | 365 days | Invoice staleness limit |
| Trusted providers | 10 | Maximum provider list size |

## On-Chain Integration

The circuit outputs `PublicOutput` which the Soroban verifier contract (`../verifier/`) checks:

```rust
struct PublicOutput {
    invoice_hash: Field,           // Pedersen commitment to invoice data (domain-separated)
    loan_amount: u64,              // Computed loan in cents
    provider_response_hash: Field, // Provider + response + TLS signature binding
    nullifier: Field,              // Double-spend prevention
    is_valid: bool,                // Circuit validation passed
}
```

The nullifier is stored on-chain to prevent the same invoice from being factored twice.

## Security Model

- **Private inputs**: Invoice details, debtor identity, zkTLS session data -- never revealed on-chain
- **Public outputs**: Only commitments and the loan amount are public
- **Nullifier scheme**: `Pedersen(invoice_id || nullifier_seed)` -- unique per invoice
- **Provider trust**: Only invoices from trusted API providers pass verification
- **LTV enforcement**: Loan amount cannot exceed the configured LTV ratio
- **Domain separation**: Commitment includes a version tag (v1) to prevent hash collisions across protocol upgrades
- **Signature binding**: The TLS signature is hashed into the provider-response commitment, preventing signature substitution