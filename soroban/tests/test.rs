#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal};

use mock_usdc::MockUsdc;
use aero_contract::types::{Asset, LoanStatus};
use aero_contract::AeroContract;

// --- Test Helpers ---

/// Builds a combined proof bytes containing mock proof data + 256-byte public inputs.
/// The verifier expects: [proof_part (N-256 bytes)] + [public_inputs (256 bytes)].
fn build_combined_proof(
    env: &Env,
    proof_tag: u8,
    loan_amount: i128,
    ltv_bps: u32,
    interest_bps: u32,
    invoice_hash_byte: u8,
    nullifier_byte: u8,
) -> Bytes {
    // Mock proof part: 128 bytes with a tag byte repeated
    let proof_len: u32 = 128;
    let mut combined: Vec<u8> = Vec::new();
    for _i in 0..proof_len {
        combined.push(proof_tag);
    }

    // Public inputs: 256 bytes (8 fields x 32 bytes each, little-endian)
    // Field 0: invoice_hash (32 bytes)
    for _i in 0..32 {
        combined.push(invoice_hash_byte);
    }
    // Field 1: loan_amount (u64 in first 8 bytes, rest zeros)
    for i in 0..32u32 {
        let byte = if i < 16 { ((loan_amount >> (i * 8)) & 0xFF) as u8 } else { 0 };
        combined.push(byte);
    }
    // Field 2: provider_response_hash (32 bytes)
    for _i in 0..32 {
        combined.push(0x22u8);
    }
    // Field 3: nullifier (32 bytes)
    for _i in 0..32 {
        combined.push(nullifier_byte);
    }
    // Field 4: ltv_bps (u32 in first 4 bytes, rest zeros)
    for i in 0..32u32 {
        let byte = if i < 4 { ((ltv_bps >> (i * 8)) & 0xFF) as u8 } else { 0 };
        combined.push(byte);
    }
    // Field 5: interest_bps (u32 in first 4 bytes, rest zeros)
    for i in 0..32u32 {
        let byte = if i < 4 { ((interest_bps >> (i * 8)) & 0xFF) as u8 } else { 0 };
        combined.push(byte);
    }
    // Fields 6, 7: reserved (zeros)
    for _i in 0..64 {
        combined.push(0u8);
    }

    Bytes::from_slice(env, &combined)
}

fn deploy_mock_usdc(env: &Env, admin: &Address) -> Address {
    let contract_id = env.register(MockUsdc, ());
    let _result: soroban_sdk::Val = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(env, "init"),
        soroban_sdk::vec![env, admin.to_val()],
    );
    contract_id
}

fn deploy_aero(env: &Env, admin: &Address, usdc_address: &Address) -> Address {
    let contract_id = env.register(AeroContract, ());
    let _result: soroban_sdk::Val = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(env, "init"),
        soroban_sdk::vec![env, admin.to_val(), usdc_address.to_val()],
    );
    contract_id
}

#[test]
fn test_mock_usdc_mint_and_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);

    let mint_amount: i128 = 1_000_000_000_000i128;
    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), user.to_val(), mint_amount.into_val(&env)],
    );

    let bal: i128 = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "balance"),
        soroban_sdk::vec![&env, user.to_val()],
    );
    assert_eq!(bal, mint_amount);
}

#[test]
fn test_init_and_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let stored_admin: Address = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "get_admin"),
        soroban_sdk::vec![&env],
    );
    assert_eq!(stored_admin, admin);
}

#[test]
fn test_double_init_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: soroban_sdk::Val = env.invoke_contract(
            &aero_addr,
            &soroban_sdk::Symbol::new(&env, "init"),
            soroban_sdk::vec![&env, admin.to_val(), usdc_addr.to_val()],
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_not_initialized_fails() {
    let env = Env::default();
    let contract_id = env.register(AeroContract, ());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: soroban_sdk::Val = env.invoke_contract(
            &contract_id,
            &soroban_sdk::Symbol::new(&env, "get_pool_stats"),
            soroban_sdk::vec![&env, Asset::USDC.into_val(&env)],
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_deposit_and_pool_stats() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lender = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), lender.to_val(), 1_000_000_000_000i128.into_val(&env)],
    );

    let deposit_amount: i128 = 1000 * 10_000_000i128;
    let _shares: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "deposit"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::USDC.into_val(&env), deposit_amount.into_val(&env)],
    );

    let _stats: soroban_sdk::Val = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "get_pool_stats"),
        soroban_sdk::vec![&env, Asset::USDC.into_val(&env)],
    );
}

#[test]
fn test_full_loan_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lender = Address::generate(&env);
    let borrower = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), lender.to_val(), 1_000_000_000_000i128.into_val(&env)],
    );
    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), borrower.to_val(), 100_000_000_000i128.into_val(&env)],
    );

    let deposit_amount: i128 = 1000 * 10_000_000i128;
    let _: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "deposit"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::USDC.into_val(&env), deposit_amount.into_val(&env)],
    );

    let loan_amount: i128 = 800 * 10_000_000i128;
    let vk_hash = BytesN::<32>::from_array(&env, &[0x11u8; 32]);
    let combined = build_combined_proof(&env, 0x11u8, loan_amount, 8000, 500, 0x01u8, 0x33u8);

    let loan_id: u64 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "request_loan"),
        soroban_sdk::vec![
            &env,
            borrower.to_val(),
            Asset::USDC.into_val(&env),
            combined.into_val(&env),
            vk_hash.to_val(),
        ],
    );
    assert_eq!(loan_id, 1);

    // Fast-forward 90 days to accrue interest before repaying
    env.ledger().set(LedgerInfo {
        timestamp: 1719000000 + 90 * 24 * 60 * 60,
        protocol_version: 22,
        sequence_number: 2,
        network_id: Default::default(),
        base_reserve: 0,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: 535680,
    });

    let interest = (loan_amount * 500 * (90 * 24 * 60 * 60)) / (31_557_600 * 10_000);
    let total_repay = loan_amount + interest;
    let _: soroban_sdk::Val = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "repay_loan"),
        soroban_sdk::vec![&env, borrower.to_val(), loan_id.into_val(&env), total_repay.into_val(&env)],
    );

    let _loan_after: soroban_sdk::Val = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "get_loan"),
        soroban_sdk::vec![&env, 1u64.into_val(&env)],
    );

    let _: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "withdraw"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::USDC.into_val(&env), deposit_amount.into_val(&env)],
    );
}

#[test]
fn test_liquidation_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lender = Address::generate(&env);
    let borrower = Address::generate(&env);
    let liquidator = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), lender.to_val(), 1_000_000_000_000i128.into_val(&env)],
    );
    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), borrower.to_val(), 100_000_000_000i128.into_val(&env)],
    );

    let deposit_amount: i128 = 1000 * 10_000_000i128;
    let _: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "deposit"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::USDC.into_val(&env), deposit_amount.into_val(&env)],
    );

    let loan_amount: i128 = 800 * 10_000_000i128;
    let vk_hash = BytesN::<32>::from_array(&env, &[0x22u8; 32]);
    let combined = build_combined_proof(&env, 0x22u8, loan_amount, 8000, 500, 0x02u8, 0x44u8);

    let loan_id: u64 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "request_loan"),
        soroban_sdk::vec![
            &env,
            borrower.to_val(),
            Asset::USDC.into_val(&env),
            combined.into_val(&env),
            vk_hash.to_val(),
        ],
    );

    // Fast-forward past due date
    env.ledger().set(LedgerInfo {
        timestamp: 1719000000 + 91 * 24 * 60 * 60,
        protocol_version: 22,
        sequence_number: 2,
        network_id: Default::default(),
        base_reserve: 0,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: 535680,
    });

    let _: soroban_sdk::Val = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "liquidate_loan"),
        soroban_sdk::vec![&env, liquidator.to_val(), loan_id.into_val(&env)],
    );

    let _loan: soroban_sdk::Val = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "get_loan"),
        soroban_sdk::vec![&env, loan_id.into_val(&env)],
    );
}

#[test]
fn test_repay_wrong_borrower_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lender = Address::generate(&env);
    let borrower = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), lender.to_val(), 1_000_000_000_000i128.into_val(&env)],
    );
    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), borrower.to_val(), 100_000_000_000i128.into_val(&env)],
    );

    let _: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "deposit"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::USDC.into_val(&env), (1000 * 10_000_000i128).into_val(&env)],
    );

    let loan_amount: i128 = 500 * 10_000_000i128;
    let vk_hash = BytesN::<32>::from_array(&env, &[0x55u8; 32]);
    let combined = build_combined_proof(&env, 0x55u8, loan_amount, 8000, 500, 0x05u8, 0x55u8);

    let loan_id: u64 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "request_loan"),
        soroban_sdk::vec![
            &env,
            borrower.to_val(),
            Asset::USDC.into_val(&env),
            combined.into_val(&env),
            vk_hash.to_val(),
        ],
    );

    let impostor = Address::generate(&env);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: soroban_sdk::Val = env.invoke_contract(
            &aero_addr,
            &soroban_sdk::Symbol::new(&env, "repay_loan"),
            soroban_sdk::vec![&env, impostor.to_val(), loan_id.into_val(&env), loan_amount.into_val(&env)],
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_loan_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: soroban_sdk::Val = env.invoke_contract(
            &aero_addr,
            &soroban_sdk::Symbol::new(&env, "get_loan"),
            soroban_sdk::vec![&env, 999u64.into_val(&env)],
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_xlm_deposit_and_loan() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lender = Address::generate(&env);
    let borrower = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), lender.to_val(), 1_000_000_000_000i128.into_val(&env)],
    );
    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), borrower.to_val(), 100_000_000_000i128.into_val(&env)],
    );

    let deposit_amount: i128 = 1000 * 10_000_000i128;
    let _: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "deposit"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::XLM.into_val(&env), deposit_amount.into_val(&env)],
    );

    let loan_amount: i128 = 800 * 10_000_000i128;
    let vk_hash = BytesN::<32>::from_array(&env, &[0x66u8; 32]);
    let combined = build_combined_proof(&env, 0x66u8, loan_amount, 8000, 500, 0x06u8, 0x66u8);

    let loan_id: u64 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "request_loan"),
        soroban_sdk::vec![
            &env,
            borrower.to_val(),
            Asset::XLM.into_val(&env),
            combined.into_val(&env),
            vk_hash.to_val(),
        ],
    );
    assert_eq!(loan_id, 1);
}

#[test]
fn test_nullifier_prevents_double_spend() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let lender = Address::generate(&env);
    let borrower = Address::generate(&env);
    let usdc_addr = deploy_mock_usdc(&env, &admin);
    let aero_addr = deploy_aero(&env, &admin, &usdc_addr);

    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), lender.to_val(), 1_000_000_000_000i128.into_val(&env)],
    );
    let _: soroban_sdk::Val = env.invoke_contract(
        &usdc_addr,
        &soroban_sdk::Symbol::new(&env, "mint"),
        soroban_sdk::vec![&env, admin.to_val(), borrower.to_val(), 100_000_000_000i128.into_val(&env)],
    );

    let _: i128 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "deposit"),
        soroban_sdk::vec![&env, lender.to_val(), Asset::USDC.into_val(&env), (2000 * 10_000_000i128).into_val(&env)],
    );

    let loan_amount: i128 = 500 * 10_000_000i128;
    let vk_hash = BytesN::<32>::from_array(&env, &[0x99u8; 32]);
    let combined = build_combined_proof(&env, 0x99u8, loan_amount, 8000, 500, 0x09u8, 0x99u8);

    // First loan should succeed
    let loan_id1: u64 = env.invoke_contract(
        &aero_addr,
        &soroban_sdk::Symbol::new(&env, "request_loan"),
        soroban_sdk::vec![
            &env,
            borrower.to_val(),
            Asset::USDC.into_val(&env),
            combined.into_val(&env),
            vk_hash.to_val(),
        ],
    );
    assert_eq!(loan_id1, 1);

    // Second loan with same proof should fail (nullifier already used)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: u64 = env.invoke_contract(
            &aero_addr,
            &soroban_sdk::Symbol::new(&env, "request_loan"),
            soroban_sdk::vec![
                &env,
                borrower.to_val(),
                Asset::USDC.into_val(&env),
                combined.into_val(&env),
                vk_hash.to_val(),
            ],
        );
    }));
    assert!(result.is_err());
}