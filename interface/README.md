# Aero Interface

This is the Next.js frontend for the Aero Zero-Knowledge Trade Finance & Invoice Factoring protocol. It connects to the Soroban smart contracts deployed on the Stellar Testnet, allowing businesses to factor their invoices while maintaining full privacy via ZK proofs.

## Tech Stack
- **Framework:** Next.js (App Router), TypeScript, React
- **Styling:** Tailwind CSS (v4), Headless UI
- **Animations:** Framer Motion, AOS
- **Notifications:** Sonner
- **Blockchain integration:** `@stellar/stellar-sdk`, `@stellar/freighter-api`

## Requirements
- Node.js (v18+)
- `pnpm` (recommended)
- Freighter Wallet extension installed in your browser

## Setup & Installation

1. Install dependencies:
```bash
cd interface
pnpm install
```

2. Configure environment variables. Create a `.env.local` file in the root of the `interface` directory:
```bash
NEXT_PUBLIC_MOCK_USDC_ID=CD... # Your Soroban Mock USDC Contract ID
NEXT_PUBLIC_MAIN_ID=CC... # Your Soroban Main Contract ID
```

3. Run the development server:
```bash
pnpm run dev
```

4. Open [http://localhost:3000](http://localhost:3000) with your browser to see the application.

## Directory Structure
- `src/app`: Next.js pages and routing
- `src/components`: Reusable UI components (Navbar, Modals, ThemeProvider)
- `src/hooks`: Custom React hooks (`use-wallet` for Freighter integration)
- `src/lib`: Helper functions (e.g., `zk-mock.ts` for generating the mock proof payload)

## Using the Freighter Wallet
Make sure you have [Freighter](https://www.freighter.app/) installed. Connect it via the top-right button in the navigation bar. Note: The app defaults to Stellar Testnet.
