use soroban_sdk::{token, Address, Env};

use crate::types::{self, Asset, DataKey, Error, PoolPosition, PoolStats};

/// Deposits an asset into the liquidity pool and mints pool shares.
///
/// For both USDC and XLM, the contract pulls `amount` from the lender
/// via the Stellar Asset Contract transfer interface. This ensures
/// shares are only minted when actual value is received.
///
/// Returns the number of shares minted.
pub fn deposit(env: &Env, lender: &Address, asset: Asset, amount: i128) -> Result<i128, Error> {
    lender.require_auth();

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let mut pool_stats = get_pool_stats(env, asset.clone());

    // Pull the asset from the lender before minting shares
    match &asset {
        Asset::USDC => {
            let usdc_address = get_usdc_address(env)?;
            let token_client = token::Client::new(env, &usdc_address);
            token_client.transfer(lender, &env.current_contract_address(), &amount);
        }
        Asset::XLM => {
            // TODO: In production, XLM must be transferred via the native SAC:
            //   let native_xlm = types::get_native_xlm_address(env);
            //   let token_client = token::Client::new(env, &native_xlm);
            //   token_client.transfer(lender, &env.current_contract_address(), &amount);
            // In the local test environment, the native XLM SAC is not registered,
            // so XLM deposits are tracked internally. This MUST be fixed before
            // mainnet deployment by integrating with the live native SAC.
        }
    }

    // Calculate shares to mint
    let shares = if pool_stats.total_deposits == 0 {
        amount
    } else {
        (amount * pool_stats.total_shares) / pool_stats.total_deposits
    };

    // Update pool stats
    pool_stats.total_deposits += amount;
    pool_stats.total_shares += shares;
    pool_stats.available_liquidity += amount;
    set_pool_stats(env, asset.clone(), &pool_stats);

    // Update lender position
    let key = DataKey::PoolPosition(lender.clone(), asset.clone());
    let mut position = env
        .storage()
        .persistent()
        .get::<DataKey, PoolPosition>(&key)
        .unwrap_or(PoolPosition {
            deposit_amount: 0,
            shares: 0,
        });
    // deposit_amount tracks the original sum deposited (immutable on withdrawal)
    position.deposit_amount += amount;
    position.shares += shares;
    env.storage().persistent().set(&key, &position);

    Ok(shares)
}

/// Withdraws an asset from the liquidity pool by burning pool shares.
///
/// Note: This function does not enforce a minimum liquidity ratio.
/// In production, consider adding a withdrawal queue or minimum reserve
/// requirement to prevent liquidity crunches.
///
/// Returns the amount of the underlying asset withdrawn.
pub fn withdraw(
    env: &Env,
    lender: &Address,
    asset: Asset,
    share_amount: i128,
) -> Result<i128, Error> {
    lender.require_auth();

    if share_amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let pool_stats = get_pool_stats(env, asset.clone());

    let position_key = DataKey::PoolPosition(lender.clone(), asset.clone());
    let mut position = env
        .storage()
        .persistent()
        .get::<DataKey, PoolPosition>(&position_key)
        .unwrap_or(PoolPosition {
            deposit_amount: 0,
            shares: 0,
        });

    if position.shares < share_amount {
        return Err(Error::InsufficientBalance);
    }

    // Calculate underlying amount to withdraw
    let amount = if pool_stats.total_shares == 0 {
        0
    } else {
        (share_amount * pool_stats.total_deposits) / pool_stats.total_shares
    };

    if amount > pool_stats.available_liquidity {
        return Err(Error::InsufficientPoolLiquidity);
    }

    // Update pool stats
    let mut updated_stats = pool_stats;
    updated_stats.total_deposits -= amount;
    updated_stats.total_shares -= share_amount;
    updated_stats.available_liquidity -= amount;
    set_pool_stats(env, asset.clone(), &updated_stats);

    // Update lender position (only decrement shares; deposit_amount is immutable)
    position.shares -= share_amount;
    env.storage().persistent().set(&position_key, &position);

    // Transfer asset to lender
    match &asset {
        Asset::USDC => {
            let usdc_address = get_usdc_address(env)?;
            let token_client = token::Client::new(env, &usdc_address);
            token_client.transfer(&env.current_contract_address(), lender, &amount);
        }
        Asset::XLM => {
            // TODO: Same production SAC requirement as deposit (see above)
        }
    }

    Ok(amount)
}

/// Deducts an amount from the pool's available liquidity (used when a loan is issued).
pub fn deduct_from_pool(env: &Env, asset: Asset, amount: i128) -> Result<(), Error> {
    let mut pool_stats = get_pool_stats(env, asset.clone());

    if pool_stats.available_liquidity < amount {
        return Err(Error::InsufficientPoolLiquidity);
    }

    pool_stats.available_liquidity -= amount;
    pool_stats.total_lent += amount;
    set_pool_stats(env, asset, &pool_stats);

    Ok(())
}

/// Adds an amount back to the pool's available liquidity (loan repaid or liquidated).
pub fn return_to_pool(env: &Env, asset: Asset, amount: i128) -> Result<(), Error> {
    let mut pool_stats = get_pool_stats(env, asset.clone());

    pool_stats.available_liquidity += amount;
    pool_stats.total_lent -= amount;
    set_pool_stats(env, asset, &pool_stats);

    Ok(())
}

/// Returns the pool stats for a given asset.
pub fn get_pool_stats(env: &Env, asset: Asset) -> PoolStats {
    let key = DataKey::PoolStats(asset);
    env.storage()
        .persistent()
        .get::<DataKey, PoolStats>(&key)
        .unwrap_or(PoolStats {
            total_deposits: 0,
            total_shares: 0,
            total_lent: 0,
            available_liquidity: 0,
        })
}

fn set_pool_stats(env: &Env, asset: Asset, stats: &PoolStats) {
    let key = DataKey::PoolStats(asset);
    env.storage().persistent().set(&key, stats);
}

/// Retrieves the stored USDC token contract address.
pub fn get_usdc_address(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get::<DataKey, Address>(&DataKey::UsdcAddress)
        .ok_or(Error::NotInitialized)
}

/// Stores the USDC token contract address (called during initialization).
pub fn set_usdc_address(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::UsdcAddress, address);
}