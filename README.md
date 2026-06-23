# Aero

![aero](https://raw.githubusercontent.com/limalime/limalime/refs/heads/main/images/aero.png)

**Aero** is a privacy-preserving Real-World Asset (RWA) lending protocol built on the Stellar network. It allows SMEs, Web3 agencies, and freelancers to borrow against unpaid real-world invoices while keeping their business data.

By combining **zkTLS** (to verify Web2 API data authenticity) with **Noir** (to generate off-chain privacy proofs) and **Stellar Soroban**, Aero bridges the multi-trillion-dollar traditional invoice factoring market with DeFi liquidity—without exposing sensitive client lists or pricing to the public blockchain.

> **Built for:** [Stellar Hacks: Real-World ZK](https://dorahacks.io/hackathon/stellar-hacks-zk/detail)  

---

## The Problem: Privacy vs. Liquidity
Trade finance (invoice factoring) is a massive industry, but it faces a critical bottleneck when moving to Web3:
1. **The SME Dilemma:** Small businesses desperately need liquidity against Net-60/Net-90 invoices. However, they refuse to tokenize invoices on public blockchains because doing so exposes their client lists, pricing models, and sales volumes to competitors.
2. **The DeFi Limitation:** Traditional DeFi requires over-collateralization. It cannot natively underwrite real-world cash flows because it cannot verify private Web2 accounting data without exposing it.
3. **The "Garbage In, Garbage Out" Risk:** Simply uploading a CSV of invoices to a ZK circuit doesn't work, as users can easily forge local files.

## The Aero Solution
Aero solves this by creating a cryptographic tunnel between private Web2 accounting software (Stripe, Xero, Gusto) and Web3 DeFi liquidity pools.

### How It Works (The Architecture)
1. **Web2 Authentication (zkTLS):** Users connect their accounting APIs via a secure zkTLS tunnel. The protocol cryptographically stamps the data using the official server's TLS certificate, proving the data came directly from the source and was not forged locally.
2. **Off-Chain Proof Generation (Noir):** The authenticated data is processed inside a Noir ZK circuit. The circuit extracts the risk parameters (e.g., "Invoice is > $10k", "Due in 30 days", "Client is Tier-1") and generates an UltraHonk proof. The actual client name and exact amount remain hidden.
3. **On-Chain Verification (Stellar Soroban):** The Soroban smart contract verifies the ZK Proof using **Stellar's new Protocol 25/26 BN254 host functions** (multi-scalar multiplication). Upon successful verification, the contract instantly releases USDC to the borrower and mints a Loan NFT.

---

## Repository Structure

This is a monorepo containing the ZK circuits, smart contracts, cryptographic verifier, and the frontend interface. For detailed setup, testing, and deployment instructions, please refer to the `README.md` inside each specific directory.

```text
aero/
├── circuits/        # Noir ZK circuits
├── soroban/         # Main Stellar Smart Contracts
├── verifier/        # Cryptographic Verifier Library
├── interface/       # Next.js Frontend
└── README.md       
```

### Directory Guide
* **[circuits/](./circuits/README.md):** Contains the Noir (`.nr`) source code. Learn how to compile the circuits, generate the Verification Key (VK), and run local ZK tests.
* **[soroban/](./soroban/README.md):** Contains the Rust Soroban contracts. Learn how to build the WASM files, run integration tests, and deploy the Mock USDC and Main Protocol to the Stellar Testnet.
* **[verifier/](./verifier/README.md):** Contains the Rust library that bridges the Noir proofs to Soroban using Stellar's BN254 host functions. 
* **[interface/](./interface/README.md):** Contains the Next.js frontend. Learn how to configure your `.env.local` with live Testnet Contract IDs and run the local development server.

---

## Prerequisites

To build and test the entire Aero monorepo locally, ensure you have the following installed:
* **Rust & Cargo:** (Required for Soroban and Verifier crates)
* **Stellar CLI:** (Required for building WASM and testnet deployment)
* **Noir (Nargo):** (Required for compiling the ZK circuits)
* **Node.js (v18+) & npm/pnpm:** (Required for the Next.js frontend)

## Quick Start

### Clone this repository 
```bash
git clone https://github.com/limalime/aero-monorepo.git && cd aero-monorepo
```

### 1. Build the Smart Contracts & Verifier
```bash
cd soroban
cargo clean
stellar contract build
```

### 2. Compile the ZK Circuits
```bash
cd circuits
nargo compile
```

### 3. Run the Frontend
```bash
cd interface
npm install
# Add your .env.local file with Testnet Contract IDs
npm run dev
```

---

## Risk Management & Security
Aero employs a three-tiered defense system for under-collateralized RWA lending:
1. **Skin-in-the-Game (Performance Bond):** Borrowers lock a 10% margin in XLM/USDC. If they default, this margin is instantly liquidated to compensate lenders.
2. **Cryptographic Notice of Assignment:** Generating the ZK proof requires signing a legal hash that transfers the collection rights of the invoice to the Aero Liquidity Pool.
3. **Persistent Nullifier Registry:** To prevent double-spending (factoring the same invoice twice), the protocol stores ZK nullifiers in Soroban's `persistent` storage tier with an extended TTL, ensuring the registry outlives the contract instance.

## License
This project is licensed under the MIT License - see the [LICENSE.md](./LICENSE.md) file for details.
