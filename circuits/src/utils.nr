// Protocol-wide constants
// Global protocol cap on invoice face value (separate from a lender's loan cap)
pub global MAX_INVOICE_AMOUNT: u64 = 100_000_000;
pub global MAX_LTV_BPS: u32 = 10_000;
// Global protocol minimum invoice amount; individual lenders may set higher
// minimums in their LendingParams (e.g. tests use 100_000 for tighter policies)
pub global MIN_INVOICE_AMOUNT: u64 = 10_000;
pub global MAX_INVOICE_AGE_DAYS: u32 = 365;
pub global SECONDS_PER_DAY: u64 = 86_400;
pub global HASH_INPUT_SIZE: u32 = 8;
pub global TRUSTED_PROVIDER_COUNT: u32 = 10;
pub global HASH_BYTE_LEN: u32 = 32;

// Currency identifiers
pub global CURRENCY_USD: u8 = 0;
pub global CURRENCY_EUR: u8 = 1;
pub global CURRENCY_GBP: u8 = 2;

/// Pedersen hash over a fixed-size array of Fields.
/// Produces a single Field digest that serves as an invoice commitment.
pub fn hash_fields(inputs: [Field; HASH_INPUT_SIZE]) -> Field {
    std::hash::pedersen_hash(inputs)
}

/// Converts a [u8; 32] byte array into a Field by interpreting it
/// as a little-endian 256-bit integer. This is safe because Field
/// modulus exceeds 2^256 for BN254.
pub fn bytes32_to_field(bytes: [u8; HASH_BYTE_LEN]) -> Field {
    let mut result: Field = 0;
    let base: Field = 256;
    let mut multiplier: Field = 1;

    for i in 0..HASH_BYTE_LEN {
        result = result + (bytes[i] as Field) * multiplier;
        multiplier = multiplier * base;
    }
    result
}

/// Converts a u64 to a Field.
pub fn u64_to_field(value: u64) -> Field {
    value as Field
}

/// Combines two Field values into a single Field via Pedersen.
pub fn hash_pair(left: Field, right: Field) -> Field {
    let mut inputs = [0; HASH_INPUT_SIZE];
    inputs[0] = left;
    inputs[1] = right;
    std::hash::pedersen_hash(inputs)
}

/// Computes the loan amount from invoice amount and LTV in basis points.
/// loan = amount * ltv_bps / 10_000
pub fn compute_loan_amount(amount: u64, ltv_bps: u32) -> u64 {
    let numerator: u64 = amount * (ltv_bps as u64);
    numerator / 10_000
}

/// Verifies that a currency code is valid.
pub fn is_valid_currency(currency: u8) -> bool {
    (currency == CURRENCY_USD) | (currency == CURRENCY_EUR) | (currency == CURRENCY_GBP)
}

/// Verifies that an amount is within the acceptable range for lending.
pub fn is_valid_amount(amount: u64) -> bool {
    (amount >= MIN_INVOICE_AMOUNT) & (amount <= MAX_INVOICE_AMOUNT)
}

/// Verifies that the due date has not yet passed.
pub fn is_not_expired(due_date: u64, current_time: u64) -> bool {
    due_date > current_time
}

/// Verifies that the invoice age (from issuance to now) is within limits.
/// Uses addition-based comparison to avoid subtraction underflow in ACIR.
pub fn is_within_age_limit(issuance_date: u64, current_time: u64, max_age_days: u32) -> bool {
    let max_age_seconds: u64 = (max_age_days as u64) * SECONDS_PER_DAY;
    let deadline: u64 = issuance_date + max_age_seconds;
    (current_time >= issuance_date) & (current_time <= deadline)
}

/// Verifies that issuance_date < due_date (sanity check).
pub fn is_chronological(issuance_date: u64, due_date: u64) -> bool {
    issuance_date < due_date
}

/// Checks if two 32-byte arrays are equal.
pub fn bytes32_eq(a: [u8; HASH_BYTE_LEN], b: [u8; HASH_BYTE_LEN]) -> bool {
    let mut eq = true;
    for i in 0..HASH_BYTE_LEN {
        eq = eq & (a[i] == b[i]);
    }
    eq
}

/// Checks if a 32-byte array is all zeros (empty provider slot).
pub fn bytes32_is_zero(bytes: [u8; HASH_BYTE_LEN]) -> bool {
    let mut is_zero = true;
    for i in 0..HASH_BYTE_LEN {
        is_zero = is_zero & (bytes[i] == 0);
    }
    is_zero
}