#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Balance(Address),
    Admin,
    TotalSupply,
}

/// Mock USDC token for testing Aero protocol integration.
///
/// This is a minimal token implementation that supports:
/// - mint: Create new tokens (admin only)
/// - transfer: Move tokens between addresses
/// - balance: Query token balance
///
/// Used in integration tests to simulate USDC deposits and loan payouts.
#[contract]
pub struct MockUsdc;

#[contractimpl]
impl MockUsdc {
    /// Initializes the mock token with an admin.
    pub fn init(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalSupply, &0i128);
    }

    /// Mints new tokens to the specified address. Admin only.
    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        assert_eq!(admin, stored_admin, "unauthorized mint");
        assert!(amount > 0, "amount must be positive");

        let key = DataKey::Balance(to.clone());
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(current + amount));

        let supply: i128 = env.storage().instance().get(&DataKey::TotalSupply).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(supply + amount));
    }

    /// Transfers tokens from the caller to the recipient.
    /// The caller must authorize the transfer.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "amount must be positive");

        let from_key = DataKey::Balance(from.clone());
        let from_balance: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
        assert!(from_balance >= amount, "insufficient balance");

        let to_key = DataKey::Balance(to.clone());
        let to_balance: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);

        env.storage()
            .persistent()
            .set(&from_key, &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&to_key, &(to_balance + amount));
    }

    /// Returns the token balance of the given address.
    pub fn balance(env: Env, id: Address) -> i128 {
        let key = DataKey::Balance(id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Returns the token name.
    pub fn name(env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, "USDC")
    }

    /// Returns the token symbol.
    pub fn symbol(env: Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(&env, "USDC")
    }

    /// Returns the token decimals.
    pub fn decimals(env: Env) -> u32 {
        7
    }

    /// Returns the total supply.
    pub fn total_supply(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalSupply).unwrap_or(0)
    }
}