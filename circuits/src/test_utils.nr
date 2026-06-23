// Unit tests for the utils module.
// Tests byte-to-field conversion, hashing, and helper functions.

use crate::utils;

#[test]
fn test_bytes32_to_field_roundtrip() {
    let bytes = [0x42; 32];
    let field = utils::bytes32_to_field(bytes);
    assert(field != 0);
}

#[test]
fn test_bytes32_to_field_zero() {
    let bytes = [0; 32];
    let field = utils::bytes32_to_field(bytes);
    assert(field == 0);
}

#[test]
fn test_hash_pair_deterministic() {
    let left: Field = 12345;
    let right: Field = 67890;
    let h1 = utils::hash_pair(left, right);
    let h2 = utils::hash_pair(left, right);
    assert(h1 == h2);
}

#[test]
fn test_hash_pair_differs_on_order() {
    let a: Field = 111;
    let b: Field = 222;
    let h_ab = utils::hash_pair(a, b);
    let h_ba = utils::hash_pair(b, a);
    assert(h_ab != h_ba);
}

#[test]
fn test_compute_loan_amount_80pct() {
    let loan = utils::compute_loan_amount(1_000_000, 8_000);
    assert(loan == 800_000);
}

#[test]
fn test_compute_loan_amount_50pct() {
    let loan = utils::compute_loan_amount(1_000_000, 5_000);
    assert(loan == 500_000);
}

#[test]
fn test_compute_loan_amount_100pct() {
    let loan = utils::compute_loan_amount(1_000_000, 10_000);
    assert(loan == 1_000_000);
}

#[test]
fn test_is_valid_currency() {
    assert(utils::is_valid_currency(utils::CURRENCY_USD));
    assert(utils::is_valid_currency(utils::CURRENCY_EUR));
    assert(utils::is_valid_currency(utils::CURRENCY_GBP));
    assert(!utils::is_valid_currency(99));
    assert(!utils::is_valid_currency(255));
}

#[test]
fn test_is_valid_amount() {
    assert(utils::is_valid_amount(100_000));
    assert(utils::is_valid_amount(10_000_000));
    assert(utils::is_valid_amount(100_000_000));
    assert(!utils::is_valid_amount(5_000));
    assert(!utils::is_valid_amount(200_000_000));
}

#[test]
fn test_is_not_expired() {
    assert(utils::is_not_expired(1_000_000, 500_000));
    assert(!utils::is_not_expired(500_000, 1_000_000));
    assert(!utils::is_not_expired(1_000_000, 1_000_000));
}

#[test]
fn test_is_within_age_limit() {
    assert(utils::is_within_age_limit(100, 100 + 86_400, 90));
    let ninety_one_days: u64 = 91 * 86_400;
    assert(!utils::is_within_age_limit(100, 100 + ninety_one_days, 90));
}

#[test]
fn test_is_chronological() {
    assert(utils::is_chronological(100, 200));
    assert(!utils::is_chronological(200, 100));
    assert(!utils::is_chronological(100, 100));
}

#[test]
fn test_bytes32_eq() {
    let a = [1; 32];
    let b = [1; 32];
    let c = [2; 32];
    assert(utils::bytes32_eq(a, b));
    assert(!utils::bytes32_eq(a, c));
}

#[test]
fn test_bytes32_is_zero() {
    let zero = [0; 32];
    let non_zero = [1; 32];
    assert(utils::bytes32_is_zero(zero));
    assert(!utils::bytes32_is_zero(non_zero));
}