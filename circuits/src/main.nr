mod utils;
mod invoice;
mod zktls;
mod lending;
mod test_utils;
mod test_integration;

use crate::invoice::InvoiceData;
use crate::zktls::ZkTlsProof;
use crate::lending::LendingParams;

/// Public output of the Aero circuit.
/// These values are revealed on-chain; the invoice details remain private.
pub struct PublicOutput {
    pub invoice_hash: Field,
    pub loan_amount: u64,
    pub provider_response_hash: Field,
    pub nullifier: Field,
    pub is_valid: bool,
}

/// Wrapper struct required because Noir main() cannot accept generic array lengths.
/// Wraps the fixed-size trusted provider array for the circuit entry point.
pub struct TrustedProviderList {
    pub providers: [[u8; 32]; 10],
}

/// Main Aero ZK circuit.
///
/// Takes private invoice data and a zkTLS proof, along with public lending
/// parameters and a trusted provider list. Produces a public output that:
/// - Commits to the invoice without revealing it
/// - Reveals the computed loan amount
/// - Provides a nullifier against double-spending
/// - Links the proof to the API provider
///
/// The Soroban verifier contract checks this proof against the public outputs.
pub fn main(
    // Private inputs
    invoice: InvoiceData,
    zktls_proof: ZkTlsProof,
    nullifier_seed: [u8; 32],
    // Public inputs
    lending_params: LendingParams,
    trusted_providers: TrustedProviderList,
) -> pub PublicOutput {
    // Validate lending parameters are internally consistent
    assert(lending_params.validate());

    // Enforce lending rules; this also validates the invoice internally
    let loan_amount = lending_params.enforce(invoice);

    // Verify the zkTLS proof: provider authorization and commitment integrity
    let proof_valid = zktls_proof.verify(trusted_providers.providers);
    assert(proof_valid);

    // Compute the invoice commitment (on-chain reference without revealing data)
    let invoice_hash = invoice.commitment();

    // Compute the provider-response binding hash
    let provider_response_hash = zktls_proof.provider_response_hash();

    // Compute the nullifier to prevent double-spending
    // nullifier = Pedersen(invoice_id || nullifier_seed)
    let id_field = utils::bytes32_to_field(invoice.get_id());
    let seed_field = utils::bytes32_to_field(nullifier_seed);
    let nullifier = utils::hash_pair(id_field, seed_field);

    PublicOutput {
        invoice_hash,
        loan_amount,
        provider_response_hash,
        nullifier,
        is_valid: true,
    }
}

#[test]
fn test_full_circuit_happy_path() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 5_000_000,  // $50,000.00
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xAA; 32],
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; 64],
    };

    let nullifier_seed = [0xEE; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xAA; 32];
    let trusted_providers = TrustedProviderList { providers };

    let output = main(
        invoice,
        zktls_proof,
        nullifier_seed,
        lending_params,
        trusted_providers,
    );

    assert(output.is_valid);
    // 80% of $50,000 = $40,000 = 4,000,000 cents
    assert(output.loan_amount == 4_000_000);
}

#[test]
fn test_circuit_computes_different_nullifiers() {
    let invoice1 = InvoiceData {
        invoice_id: [1; 32],
        amount: 5_000_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let invoice2 = InvoiceData {
        invoice_id: [2; 32],  // Different ID
        amount: 5_000_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xAA; 32],
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; 64],
    };

    let nullifier_seed = [0xEE; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xAA; 32];
    let trusted_providers = TrustedProviderList { providers };

    let out1 = main(invoice1, zktls_proof, nullifier_seed, lending_params, trusted_providers);
    let out2 = main(invoice2, zktls_proof, nullifier_seed, lending_params, trusted_providers);

    // Different invoices should produce different nullifiers
    assert(out1.nullifier != out2.nullifier);
}

#[test]
fn test_circuit_same_nullifier_for_same_invoice() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 5_000_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };

    let zktls_proof = ZkTlsProof {
        provider_id: [0xAA; 32],
        session_hash: [0xBB; 32],
        response_commitment: [0xCC; 32],
        tls_signature: [0xDD; 64],
    };

    let nullifier_seed = [0xEE; 32];

    let lending_params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };

    let mut providers = [[0; 32]; 10];
    providers[0] = [0xAA; 32];
    let trusted_providers = TrustedProviderList { providers };

    let out1 = main(invoice, zktls_proof, nullifier_seed, lending_params, trusted_providers);
    let out2 = main(invoice, zktls_proof, nullifier_seed, lending_params, trusted_providers);

    // Same invoice + same seed = same nullifier (deterministic)
    assert(out1.nullifier == out2.nullifier);
}