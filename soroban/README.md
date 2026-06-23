# Aero Soroban Contracts

Stellar smart contracts for the Aero Protocol

## Architecture

```
soroban/
├── Cargo.toml          # Workspace + main contract manifest
├── src/
│   ├── lib.rs          # Main contract: AeroContract with all public functions
│   ├── types.rs        # Data types: Asset, LoanData, PoolStats, errors (19 variants), storage keys
│   ├── pool.rs         # Liquidity pool: deposit, withdraw, share tracking
│   ├── loan.rs         # Loan lifecycle: request, repay, liquidate, interest calc
│   └── verifier.rs     # ZK proof verifier placeholder + nullifier extraction
├── mock-usdc/          # Standalone mock USDC workspace member (for actual deployment)
│   ├── Cargo.toml
│   └── src/lib.rs
├── tests/
│   └── test.rs         # Integration tests: full lifecycle, multi-asset, liquidation
└── README.md
```

## Contract Overview: AeroContract

The `AeroContract` manages the entire protocol lifecycle:

| Function | Description |
|----------|-------------|
| `init(admin, usdc_address)` | One-time initialization; sets admin and USDC token address |
| `deposit(lender, asset, amount)` | Deposit XLM or USDC into the liquidity pool; mints shares |
| `withdraw(lender, asset, share_amount)` | Burn pool shares and withdraw underlying asset |
| `get_pool_stats(asset)` | Query pool statistics for an asset |
| `get_pool_position(lender, asset)` | Query a lender's position for an asset |
| `request_loan(borrower, invoice_hash, amount, asset, ltv_bps, interest_bps, proof_bytes)` | Create a loan backed by a ZK-verified invoice; requires 10% bond. LTV and interest rate are parameterized (Phase Three: extracted from proof outputs) |
| `repay_loan(borrower, loan_id, amount)` | Repay a loan with interest; returns bond on full repayment |
| `liquidate_loan(liquidator, loan_id)` | Liquidate a past-due loan; slashes borrower's bond |
| `get_loan(loan_id)` | Query full loan data |
| `get_loan_count()` | Query total number of loans created |
| `verify_proof(proof_bytes)` | ZK proof verification placeholder (Phase Three) |

## Multi-Asset Support

The protocol natively supports two asset types:

- **XLM**: Stellar's native asset. Transfers use the Stellar Asset Contract interface with the well-known native XLM address (`CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUF34TQ6NPVHSZ2IXIZQNQAL`).
- **USDC**: A Stellar Asset Contract (SAC) token. The token address is configured during `init()`.

All pool and loan functions accept an `Asset` enum (`Asset::XLM` or `Asset::USDC`) to route logic correctly.

## Loan Lifecycle

```
Lender deposits → Pool funded
                      ↓
Borrower requests loan (posts ZK proof + 10% bond)
                      ↓
Contract verifies proof placeholder → issues loan
                      ↓
Loan active (accruing interest at configured rate)
         ↓                        ↓
    Borrower repays          Past due date
    principal + interest         ↓
         ↓                  Liquidator calls liquidate
    Bond returned               ↓
    NFT burned              Bond slashed → pool
```

### Economic Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| Skin-in-the-game | 10% | Bond posted by borrower (basis points) |
| Loan-to-value (LTV) | 80% | Max loan as percentage of invoice value |
| Interest rate | 5% APR | Simple interest calculated per-second |
| Loan term | 90 days | Standard loan duration |

## Verifier Integration (Phase Three)

The `verify_proof()` function in `src/verifier.rs` is a placeholder that always returns `true`. In Phase Three, the `verifier/` crate will provide:

- Actual Groth16/ACIR proof verification
- Nullifier extraction and on-chain double-spend registry
- Public input validation against circuit outputs

## Testing

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Run integration tests
cargo test
```

Tests cover:
- Mock USDC mint, transfer, and balance
- Full loan lifecycle (deposit → borrow → repay → withdraw)
- Liquidation of past-due loans
- Authorization checks (wrong borrower, double init)
- XLM and USDC multi-asset flows
- Proof verification placeholder

### Mock USDC Token

The mock USDC contract (`src/mock.rs`) is used in tests to simulate USDC token transfers. It supports:
- `init(admin)` -- Initialize with admin
- `mint(admin, to, amount)` -- Mint tokens (admin only)
- `transfer(from, to, amount)` -- Transfer tokens between addresses
- `balance(id)` -- Query balance

A standalone version is also available in `mock-usdc/` for external deployment.

## Deploying to Testnet

```bash
# Build both contracts
stellar contract build

# Deploy Mock USDC (optional; use real USDC in production)
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/mock_usdc.wasm \
  --source alice \
  --network testnet

# Deploy Aero contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/aero_contract.wasm \
  --source alice \
  --network testnet

# Initialize
stellar contract invoke \
  --id <AERO_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- init --admin alice --usdc_address <USDC_CONTRACT_ID>
```

## Storage Layout

| Key | Type | Scope | Description |
|-----|------|-------|-------------|
| `DataKey::Admin` | `Address` | Instance | Protocol administrator |
| `DataKey::UsdcAddress` | `Address` | Instance | USDC token contract address |
| `DataKey::LoanCounter` | `u64` | Instance | Auto-incrementing loan ID counter |
| `DataKey::Initialized` | `bool` | Instance | Init guard flag |
| `DataKey::Loan(id)` | `LoanData` | Persistent | Per-loan data (includes accrued_interest for partial repayments) |
| `DataKey::PoolStats(asset)` | `PoolStats` | Persistent | Pool stats per asset |
| `DataKey::PoolPosition(lender, asset)` | `PoolPosition` | Persistent | Lender position per asset (deposit_amount immutable) |
| `DataKey::NullifierUsed(hash)` | `BytesN<32>` | Instance | Double-spend prevention registry |