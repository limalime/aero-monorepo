// Integration tests for the full Aero circuit.
// These tests exercise the complete circuit flow across all modules.

use crate::utils;
use crate::invoice::InvoiceData;
use crate::zktls::ZkTlsProof;
use crate::lending::LendingParams;

#[test]
fn test_integration_happy_path_large_invoice() {
    let invoice = InvoiceData {
        invoice_id: [0x0A; 32],
        amount: 50_000_000,  // $500,000.00
        debtor_hash: [0x0B; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_729_000_000,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xCA; 32],
        session_hash: [0xFE; 32],
        response_commitment: [0xBA; 32],
        tls_signature: [0xBE; 64],
    };

    let nullifier_seed = [0x11; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 7_500,  // 75% LTV
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xCA; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let output = crate::main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
    // 75% of $500,000 = $375,000 = 37,500,000 cents
    assert(output.loan_amount == 37_500_000);
    assert(output.invoice_hash != 0);
    assert(output.provider_response_hash != 0);
    assert(output.nullifier != 0);
}

#[test]
fn test_integration_eur_invoice() {
    let invoice = InvoiceData {
        invoice_id: [0x21; 32],
        amount: 250_000,  // 2,500.00 EUR
        debtor_hash: [0x22; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_EUR,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xCA; 32],
        session_hash: [0xFE; 32],
        response_commitment: [0xBA; 32],
        tls_signature: [0xBE; 64],
    };

    let nullifier_seed = [0x22; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 50_000,
        max_invoice_age_days: 60,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xCA; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let output = crate::main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
    // 80% of 2,500 = 2,000 EUR = 200,000 cents
    assert(output.loan_amount == 200_000);
}

#[test]
fn test_integration_gbp_invoice() {
    let invoice = InvoiceData {
        invoice_id: [0x31; 32],
        amount: 1_000_000,  // 10,000.00 GBP
        debtor_hash: [0x32; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_GBP,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xCA; 32],
        session_hash: [0xFE; 32],
        response_commitment: [0xBA; 32],
        tls_signature: [0xBE; 64],
    };

    let nullifier_seed = [0x33; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xCA; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let output = crate::main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
    // 80% of 10,000 = 8,000 GBP = 800,000 cents
    assert(output.loan_amount == 800_000);
}

#[test]
fn test_integration_min_amount_boundary() {
    let invoice = InvoiceData {
        invoice_id: [0x41; 32],
        amount: 100_000,  // Exactly min_invoice_amount
        debtor_hash: [0x42; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xCA; 32],
        session_hash: [0xFE; 32],
        response_commitment: [0xBA; 32],
        tls_signature: [0xBE; 64],
    };

    let nullifier_seed = [0x44; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xCA; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let output = crate::main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
    // 80% of $1,000 = $800 = 80,000 cents
    assert(output.loan_amount == 80_000);
}

#[test]
fn test_integration_multiple_trusted_providers() {
    let invoice = InvoiceData {
        invoice_id: [0x51; 32],
        amount: 5_000_000,
        debtor_hash: [0x52; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };

    let target_provider = [0xDD; 32];
    let zktls_proof = ZkTlsProof {
        provider_id: target_provider,
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xEE; 64],
    };

    let nullifier_seed = [0x55; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[2] = [0xAA; 32];
    providers[4] = target_provider;
    providers[7] = [0xBB; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let output = crate::main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
    assert(output.loan_amount == 4_000_000);
}

#[test]
fn test_integration_near_due_date() {
    let invoice = InvoiceData {
        invoice_id: [0x61; 32],
        amount: 1_000_000,
        debtor_hash: [0x62; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_719_086_401,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xCA; 32],
        session_hash: [0xFE; 32],
        response_commitment: [0xBA; 32],
        tls_signature: [0xBE; 64],
    };

    let nullifier_seed = [0x66; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_086_400,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xCA; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let output = crate::main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
}

#[test]
fn test_integration_diff_invoices_diff_hashes() {
    let invoice_a = InvoiceData {
        invoice_id: [0xA1; 32],
        amount: 5_000_000,
        debtor_hash: [0xA2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let invoice_b = InvoiceData {
        invoice_id: [0xB1; 32],
        amount: 5_000_000,
        debtor_hash: [0xB2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xCA; 32],
        session_hash: [0xFE; 32],
        response_commitment: [0xBA; 32],
        tls_signature: [0xBE; 64],
    };

    let nullifier_seed = [0x77; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xCA; 32];
    let trusted_providers = crate::TrustedProviderList { providers };

    let out_a = crate::main(invoice_a, zktls_proof, nullifier_seed, lending_params, trusted_providers);
    let out_b = crate::main(invoice_b, zktls_proof, nullifier_seed, lending_params, trusted_providers);

    assert(out_a.invoice_hash != out_b.invoice_hash);
    assert(out_a.nullifier != out_b.nullifier);
}