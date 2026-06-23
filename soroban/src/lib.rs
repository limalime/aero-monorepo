#![no_std]

pub mod loan;
pub mod pool;
pub mod types;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};

use crate::types::{Asset, DataKey, Error, LoanData, PoolPosition, PoolStats};

/// Aero Protocol -- Zero-Knowledge Trade Finance & Invoice Factoring on Stellar.
///
/// This contract manages:
/// - Multi-asset liquidity pool (XLM + USDC)
/// - Loan origination backed by ZK-verified invoices (via aero-verifier crate)
/// - Loan NFT minting and lifecycle management
/// - Repayment and liquidation with 10% performance bonds
///
/// Proof verification is handled by the aero-verifier library crate (Phase Three).
#[contract]
pub struct AeroContract;

#[contractimpl]
impl AeroContract {
    /// Initializes the Aero protocol contract.
    ///
    /// Sets the admin and registers the USDC token contract address.
    /// Must be called once before any other function.
    pub fn init(env: Env, admin: Address, usdc_address: Address) -> Result<(), Error> {
        // Authorize first to avoid wasting compute on unauthorized calls
        admin.require_auth();

        // Prevent re-initialization
        if env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Initialized)
            .unwrap_or(false)
        {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UsdcAddress, &usdc_address);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::LoanCounter, &0u64);

        Ok(())
    }

    // --- Liquidity Pool ---

    /// Deposits an asset into the liquidity pool.
    ///
    /// For both XLM and USDC, the contract pulls tokens from the lender
    /// via the Stellar Asset Contract transfer interface. Shares are
    /// minted proportionally.
    ///
    /// Returns the number of pool shares minted.
    pub fn deposit(env: Env, lender: Address, asset: Asset, amount: i128) -> Result<i128, Error> {
        require_initialized(&env)?;
        pool::deposit(&env, &lender, asset, amount)
    }

    /// Withdraws an asset from the liquidity pool by burning shares.
    ///
    /// Returns the amount of underlying asset withdrawn.
    pub fn withdraw(
        env: Env,
        lender: Address,
        asset: Asset,
        share_amount: i128,
    ) -> Result<i128, Error> {
        require_initialized(&env)?;
        pool::withdraw(&env, &lender, asset, share_amount)
    }

    /// Returns pool statistics for a given asset.
    pub fn get_pool_stats(env: Env, asset: Asset) -> Result<PoolStats, Error> {
        require_initialized(&env)?;
        Ok(pool::get_pool_stats(&env, asset))
    }

    /// Returns a lender's pool position for a given asset.
    pub fn get_pool_position(
        env: Env,
        lender: Address,
        asset: Asset,
    ) -> Result<PoolPosition, Error> {
        require_initialized(&env)?;
        let key = DataKey::PoolPosition(lender, asset);
        Ok(env
            .storage()
            .persistent()
            .get::<DataKey, PoolPosition>(&key)
            .unwrap_or(PoolPosition {
                deposit_amount: 0,
                shares: 0,
            }))
    }

    // --- Loan Management ---

    /// Requests a new loan backed by a ZK-verified invoice.
    ///
    /// The borrower must provide a Noir UltraHonk proof (`proof_bytes`) that
    /// contains both the cryptographic proof and the 256-byte serialized
    /// public inputs. The Verification Key hash (`vk_hash`) identifies which
    /// circuit VK should be used for verification.
    ///
    /// All loan parameters (amount, LTV, interest rate, invoice hash) are
    /// extracted from the verified proof's public outputs. The borrower
    /// cannot arbitrarily choose loan terms.
    ///
    /// A 10% performance bond is required from the borrower.
    ///
    /// Returns the loan ID.
    pub fn request_loan(
        env: Env,
        borrower: Address,
        asset: Asset,
        proof_bytes: Bytes,
        vk_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        require_initialized(&env)?;
        loan::request_loan(&env, &borrower, asset, proof_bytes, vk_hash)
    }

    /// Repays a loan with interest.
    ///
    /// Payments are applied to principal first, then interest. On full
    /// repayment, the 10% performance bond is returned.
    pub fn repay_loan(env: Env, borrower: Address, loan_id: u64, amount: i128) -> Result<(), Error> {
        require_initialized(&env)?;
        loan::repay_loan(&env, &borrower, loan_id, amount)
    }

    /// Liquidates a loan that has passed its due date.
    ///
    /// The borrower's performance bond is slashed and returned to the pool
    /// as compensation to lenders. The loan is marked as Defaulted.
    pub fn liquidate_loan(env: Env, liquidator: Address, loan_id: u64) -> Result<(), Error> {
        require_initialized(&env)?;
        loan::liquidate_loan(&env, &liquidator, loan_id)
    }

    /// Returns the full loan data for a given loan ID.
    pub fn get_loan(env: Env, loan_id: u64) -> Result<LoanData, Error> {
        require_initialized(&env)?;
        loan::get_loan(&env, loan_id)
    }

    /// Returns the total number of loans created.
    pub fn get_loan_count(env: Env) -> Result<u64, Error> {
        require_initialized(&env)?;
        Ok(loan::get_loan_count(&env))
    }

    // --- Admin ---

    /// Returns the contract admin address.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        require_initialized(&env)?;
        env.storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }
}

/// Verifies that the contract has been initialized.
fn require_initialized(env: &Env) -> Result<(), Error> {
    if !env
        .storage()
        .instance()
        .get::<DataKey, bool>(&DataKey::Initialized)
        .unwrap_or(false)
    {
        return Err(Error::NotInitialized);
    }
    Ok(())
}