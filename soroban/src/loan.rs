use soroban_sdk::{token, Address, Bytes, BytesN, Env};

use crate::pool;
use crate::types::{Asset, DataKey, Error, LoanData, LoanStatus, SKIN_IN_GAME_BPS};

/// Requests a new loan backed by a ZK-verified invoice.
///
/// The borrower must provide a Noir UltraHonk proof (`proof_bytes`) and the
/// Verification Key hash (`vk_hash`). The proof contains both the cryptographic
/// proof and the 256-byte serialized public inputs (8 fields x 32 bytes).
///
/// All loan parameters (amount, LTV, interest rate, invoice hash, nullifier)
/// are extracted from the verified proof's public outputs. This ensures the
/// borrower cannot arbitrarily choose loan terms -- they must match the ZK
/// circuit's computed values.
///
/// A 10% performance bond is required from the borrower, calculated from the
/// verified loan amount.
///
/// Returns the loan ID.
pub fn request_loan(
    env: &Env,
    borrower: &Address,
    asset: Asset,
    proof_bytes: Bytes,
    vk_hash: BytesN<32>,
) -> Result<u64, Error> {
    borrower.require_auth();

    // Verify the ZK proof and extract all public inputs
    let public_inputs = aero_verifier::verify(env, &proof_bytes, &vk_hash)
        .map_err(|_| Error::ProofVerificationFailed)?;

    let amount = public_inputs.loan_amount;
    let ltv_bps = public_inputs.ltv_bps;
    let interest_bps = public_inputs.interest_bps;
    let invoice_hash = public_inputs.invoice_hash;
    let nullifier = public_inputs.nullifier;

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Calculate required skin-in-the-game (10% of verified loan amount)
    let required_bond = (amount * SKIN_IN_GAME_BPS) / 10000;
    if required_bond <= 0 {
        return Err(Error::InvalidSkinInGame);
    }

    // Check nullifier for double-spend prevention
    let nullifier_key = DataKey::NullifierUsed(nullifier.clone());
    if env.storage().persistent().has(&nullifier_key) {
        return Err(Error::NullifierAlreadyUsed);
    }
    env.storage().persistent().set(&nullifier_key, &true);
    env.storage().persistent().extend_ttl(&nullifier_key, 535680, 535680);

    // Deduct loan amount from pool
    pool::deduct_from_pool(env, asset.clone(), amount)?;

    // Collect skin-in-the-game bond from borrower
    collect_bond(env, borrower, &asset, required_bond)?;

    // Generate loan ID
    let loan_id = increment_loan_counter(env);

    // Set loan due date (90 days from now)
    let created_at = env.ledger().timestamp();
    let due_date = created_at + 90 * 24 * 60 * 60;

    let loan_data = LoanData {
        borrower: borrower.clone(),
        invoice_hash,
        loan_amount: amount,
        asset: asset.clone(),
        skin_in_game: required_bond,
        ltv_bps,
        interest_bps,
        status: LoanStatus::Active,
        created_at,
        due_date,
        amount_repaid: 0,
        accrued_interest: 0,
    };

    let loan_key = DataKey::Loan(loan_id);
    env.storage().persistent().set(&loan_key, &loan_data);

    // Transfer loan amount to borrower
    transfer_asset(env, &env.current_contract_address(), borrower, &asset, amount)?;

    Ok(loan_id)
}

/// Repays a loan with interest.
///
/// On first repayment, total interest is calculated and locked to prevent
/// recalculation drift across multiple partial payments. Payments are
/// applied to principal first, then interest.
///
/// On full repayment the performance bond is returned to the borrower,
/// the loan is marked as Repaid, and pool liquidity is replenished.
pub fn repay_loan(env: &Env, borrower: &Address, loan_id: u64, amount: i128) -> Result<(), Error> {
    borrower.require_auth();

    let loan_key = DataKey::Loan(loan_id);
    let mut loan = env
        .storage()
        .persistent()
        .get::<DataKey, LoanData>(&loan_key)
        .ok_or(Error::LoanNotFound)?;

    if loan.borrower != *borrower {
        return Err(Error::NotBorrower);
    }

    if loan.status != LoanStatus::Active {
        return Err(Error::LoanNotActive);
    }

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Lock total interest on first repayment to prevent recalculation drift
    if loan.accrued_interest == 0 {
        let remaining = loan.loan_amount - loan.amount_repaid;
        loan.accrued_interest = calculate_interest(
            remaining,
            loan.interest_bps,
            loan.created_at,
            env.ledger().timestamp(),
        );
    }

    let remaining_principal = loan.loan_amount - loan.amount_repaid;
    let total_owed = remaining_principal + loan.accrued_interest;

    if amount > total_owed {
        return Err(Error::InvalidAmount);
    }

    // Collect repayment from borrower
    collect_repayment(env, borrower, &loan.asset, amount)?;

    // Split payment: principal first, then interest
    let principal_paid = if amount <= remaining_principal {
        amount
    } else {
        remaining_principal
    };
    let interest_paid = amount - principal_paid;

    loan.amount_repaid += principal_paid;

    if loan.amount_repaid >= loan.loan_amount {
        // Fully repaid: return bond, mark repaid, replenish pool
        transfer_asset(
            env,
            &env.current_contract_address(),
            borrower,
            &loan.asset,
            loan.skin_in_game,
        )?;

        pool::return_to_pool(env, loan.asset.clone(), loan.loan_amount + interest_paid)?;

        loan.status = LoanStatus::Repaid;
    } else {
        // Partial repayment: return principal portion to available liquidity
        pool::return_to_pool(env, loan.asset.clone(), principal_paid)?;
    }

    env.storage().persistent().set(&loan_key, &loan);

    Ok(())
}

/// Liquidates a loan that has passed its due date.
///
/// The borrower's performance bond is slashed and returned to the pool
/// as compensation to lenders. The loan is marked as Defaulted.
pub fn liquidate_loan(env: &Env, liquidator: &Address, loan_id: u64) -> Result<(), Error> {
    liquidator.require_auth();

    let loan_key = DataKey::Loan(loan_id);
    let mut loan = env
        .storage()
        .persistent()
        .get::<DataKey, LoanData>(&loan_key)
        .ok_or(Error::LoanNotFound)?;

    if loan.status != LoanStatus::Active {
        return Err(Error::LoanNotActive);
    }

    // Check if loan is past due
    let current_time = env.ledger().timestamp();
    if current_time < loan.due_date {
        return Err(Error::LoanNotDue);
    }

    // Slash the bond: return it to pool as compensation to lenders
    pool::return_to_pool(env, loan.asset.clone(), loan.skin_in_game)?;

    loan.status = LoanStatus::Defaulted;
    env.storage().persistent().set(&loan_key, &loan);

    Ok(())
}

/// Returns the loan data for a given loan ID.
pub fn get_loan(env: &Env, loan_id: u64) -> Result<LoanData, Error> {
    let loan_key = DataKey::Loan(loan_id);
    env.storage()
        .persistent()
        .get::<DataKey, LoanData>(&loan_key)
        .ok_or(Error::LoanNotFound)
}

/// Returns the current loan counter value.
pub fn get_loan_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get::<DataKey, u64>(&DataKey::LoanCounter)
        .unwrap_or(0)
}

// --- Internal Helpers ---

fn increment_loan_counter(env: &Env) -> u64 {
    let key = DataKey::LoanCounter;
    let count: u64 = env.storage().instance().get(&key).unwrap_or(0) + 1;
    env.storage().instance().set(&key, &count);
    count
}

fn collect_bond(env: &Env, from: &Address, asset: &Asset, amount: i128) -> Result<(), Error> {
    transfer_asset(env, from, &env.current_contract_address(), asset, amount)
}

fn collect_repayment(env: &Env, from: &Address, asset: &Asset, amount: i128) -> Result<(), Error> {
    transfer_asset(env, from, &env.current_contract_address(), asset, amount)
}

/// Transfers an asset between two addresses using the SAC token interface.
fn transfer_asset(
    env: &Env,
    from: &Address,
    to: &Address,
    asset: &Asset,
    amount: i128,
) -> Result<(), Error> {
    match asset {
        Asset::USDC => {
            let usdc_addr = pool::get_usdc_address(env)?;
            let token_client = token::Client::new(env, &usdc_addr);
            token_client.transfer(from, to, &amount);
        }
        Asset::XLM => {
            // TODO: Same production SAC requirement as pool deposit (see pool.rs)
        }
    }
    Ok(())
}

/// Calculates simple interest accrued on a principal amount.
///
/// interest = principal * rate_bps * elapsed_seconds / (365.25 * 24 * 3600 * 10000)
fn calculate_interest(principal: i128, rate_bps: u32, start_time: u64, end_time: u64) -> i128 {
    if end_time <= start_time || principal <= 0 {
        return 0;
    }
    let elapsed: i128 = (end_time - start_time) as i128;
    let seconds_per_year: i128 = 31_557_600;
    (principal * (rate_bps as i128) * elapsed) / (seconds_per_year * 10000)
}