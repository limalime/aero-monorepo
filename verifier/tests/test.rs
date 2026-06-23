#![cfg(test)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env, IntoVal};

use aero_verifier::types::{Error, PublicInputs};
use aero_verifier::VerifierContract;

fn deploy_verifier(env: &Env, admin: &Address, vk_hash: BytesN<32>) -> Address {
    let contract_id = env.register(VerifierContract, ());
    let num_inputs: u32 = 8;
    let _: soroban_sdk::Val = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(env, "init"),
        soroban_sdk::vec![env, admin.to_val(), vk_hash.to_val(), num_inputs.into_val(env)],
    );
    contract_id
}

#[test]
fn test_init_and_get_vk() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vk_hash = BytesN::<32>::from_array(&env, &[0xAB; 32]);
    let contract_id = deploy_verifier(&env, &admin, vk_hash.clone());

    let vk: aero_verifier::types::VerificationKey = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(&env, "get_vk"),
        soroban_sdk::vec![&env],
    );

    assert_eq!(vk.vk_hash, vk_hash);
    assert_eq!(vk.proof_system, 0);
    assert_eq!(vk.curve, 1);
}

#[test]
fn test_double_init_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vk_hash = BytesN::<32>::from_array(&env, &[0xCD; 32]);
    let contract_id = deploy_verifier(&env, &admin, vk_hash.clone());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let num_inputs: u32 = 8;
        let _: soroban_sdk::Val = env.invoke_contract(
            &contract_id,
            &soroban_sdk::Symbol::new(&env, "init"),
            soroban_sdk::vec![&env, admin.to_val(), vk_hash.to_val(), num_inputs.into_val(&env)],
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_verify_public_input_extraction() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vk_hash = BytesN::<32>::from_array(&env, &[0xEF; 32]);
    let contract_id = deploy_verifier(&env, &admin, vk_hash);

    let mut public_inputs_bytes = [0u8; 256];

    for i in 0..32 { public_inputs_bytes[i] = 0x11u8; }
    let loan_amount: i128 = 5_000_000_000i128;
    for i in 0..16 {
        public_inputs_bytes[32 + i] = ((loan_amount >> (i * 8)) & 0xFF) as u8;
    }
    for i in 0..32 { public_inputs_bytes[64 + i] = 0x22u8; }
    for i in 0..32 { public_inputs_bytes[96 + i] = 0x33u8; }
    let ltv: u32 = 8000;
    for i in 0..4 { public_inputs_bytes[128 + i] = ((ltv >> (i * 8)) & 0xFF) as u8; }
    let int_bps: u32 = 500;
    for i in 0..4 { public_inputs_bytes[160 + i] = ((int_bps >> (i * 8)) & 0xFF) as u8; }

    let proof_bytes = Bytes::from_slice(&env, &[0xAA; 128]);
    let vk_bytes = Bytes::from_slice(&env, &[0xEF; 32]);
    let public_bytes = Bytes::from_slice(&env, &public_inputs_bytes);

    let result: PublicInputs = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(&env, "verify"),
        soroban_sdk::vec![
            &env,
            proof_bytes.into_val(&env),
            public_bytes.into_val(&env),
            vk_bytes.into_val(&env),
        ],
    );

    assert_eq!(result.loan_amount, 5_000_000_000i128);
    assert_eq!(result.ltv_bps, 8000);
    assert_eq!(result.interest_bps, 500);
    for i in 0..32 {
        assert_eq!(result.nullifier.get(i).unwrap(), 0x33u8);
    }
}

#[test]
fn test_nullifier_double_spend_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vk_hash = BytesN::<32>::from_array(&env, &[0xFE; 32]);
    let contract_id = deploy_verifier(&env, &admin, vk_hash);

    let mut public_inputs_bytes = [0u8; 256];
    let amt: i128 = 1_000_000;
    for i in 0..16 { public_inputs_bytes[32 + i] = ((amt >> (i * 8)) & 0xFF) as u8; }
    let ltv: u32 = 8000;
    for i in 0..4 { public_inputs_bytes[128 + i] = ((ltv >> (i * 8)) & 0xFF) as u8; }
    let int_bps: u32 = 500;
    for i in 0..4 { public_inputs_bytes[160 + i] = ((int_bps >> (i * 8)) & 0xFF) as u8; }

    let proof_bytes = Bytes::from_slice(&env, &[0xBB; 128]);
    let vk_bytes = Bytes::from_slice(&env, &[0xFE; 32]);
    let public_bytes = Bytes::from_slice(&env, &public_inputs_bytes);

    let _result1: PublicInputs = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(&env, "verify"),
        soroban_sdk::vec![
            &env,
            proof_bytes.into_val(&env),
            public_bytes.into_val(&env),
            vk_bytes.into_val(&env),
        ],
    );

    let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: PublicInputs = env.invoke_contract(
            &contract_id,
            &soroban_sdk::Symbol::new(&env, "verify"),
            soroban_sdk::vec![
                &env,
                proof_bytes.into_val(&env),
                public_bytes.into_val(&env),
                vk_bytes.into_val(&env),
            ],
        );
    }));
    assert!(result2.is_err());
}

#[test]
fn test_invalid_proof_format_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vk_hash = BytesN::<32>::from_array(&env, &[0xDA; 32]);
    let contract_id = deploy_verifier(&env, &admin, vk_hash);

    let short_public_bytes = Bytes::from_slice(&env, &[0u8; 64]);
    let proof_bytes = Bytes::from_slice(&env, &[0xCC; 128]);
    let vk_bytes = Bytes::from_slice(&env, &[0xDA; 32]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: PublicInputs = env.invoke_contract(
            &contract_id,
            &soroban_sdk::Symbol::new(&env, "verify"),
            soroban_sdk::vec![
                &env,
                proof_bytes.into_val(&env),
                short_public_bytes.into_val(&env),
                vk_bytes.into_val(&env),
            ],
        );
    }));
    assert!(result.is_err());
}

#[test]
fn test_verify_without_nullifier() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vk_hash = BytesN::<32>::from_array(&env, &[0xAF; 32]);
    let contract_id = deploy_verifier(&env, &admin, vk_hash);

    let mut public_inputs_bytes = [0u8; 256];
    let amt: i128 = 1_000_000;
    for i in 0..16 { public_inputs_bytes[32 + i] = ((amt >> (i * 8)) & 0xFF) as u8; }
    let ltv: u32 = 8000;
    for i in 0..4 { public_inputs_bytes[128 + i] = ((ltv >> (i * 8)) & 0xFF) as u8; }
    let int_bps: u32 = 500;
    for i in 0..4 { public_inputs_bytes[160 + i] = ((int_bps >> (i * 8)) & 0xFF) as u8; }

    let proof_bytes = Bytes::from_slice(&env, &[0xDD; 128]);
    let vk_bytes = Bytes::from_slice(&env, &[0xAF; 32]);
    let public_bytes = Bytes::from_slice(&env, &public_inputs_bytes);

    let result: PublicInputs = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(&env, "verify_without_nullifier"),
        soroban_sdk::vec![
            &env,
            proof_bytes.into_val(&env),
            public_bytes.into_val(&env),
            vk_bytes.into_val(&env),
        ],
    );
    assert_eq!(result.loan_amount, 1_000_000i128);

    let result2: PublicInputs = env.invoke_contract(
        &contract_id,
        &soroban_sdk::Symbol::new(&env, "verify_without_nullifier"),
        soroban_sdk::vec![
            &env,
            proof_bytes.into_val(&env),
            public_bytes.into_val(&env),
            vk_bytes.into_val(&env),
        ],
    );
    assert_eq!(result2.loan_amount, 1_000_000i128);
}

#[test]
fn test_not_initialized_fails() {
    let env = Env::default();
    let contract_id = env.register(VerifierContract, ());

    let proof_bytes = Bytes::from_slice(&env, &[0xEE; 128]);
    let public_bytes = Bytes::from_slice(&env, &[0u8; 256]);
    let vk_bytes = Bytes::from_slice(&env, &[0xEE; 32]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: PublicInputs = env.invoke_contract(
            &contract_id,
            &soroban_sdk::Symbol::new(&env, "verify"),
            soroban_sdk::vec![
                &env,
                proof_bytes.into_val(&env),
                public_bytes.into_val(&env),
                vk_bytes.into_val(&env),
            ],
        );
    }));
    assert!(result.is_err());
}