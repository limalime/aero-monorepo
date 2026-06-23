use crate::utils;
use crate::invoice::InvoiceData;

/// Lending parameters that govern loan issuance.
/// These are public inputs to the circuit and are set by the protocol.
pub struct LendingParams {
    pub max_loan_amount: u64,
    pub loan_to_value_bps: u32,
    pub min_invoice_amount: u64,
    pub max_invoice_age_days: u32,
    pub current_time: u64,
}

impl LendingParams {
    /// Validates that the lending parameters are internally consistent.
    /// Ensures LTV is non-zero and not above the global cap,
    /// maximums exceed minimums, and time is non-zero.
    pub fn validate(self) -> bool {
        let ltv_ok = (self.loan_to_value_bps > 0) & (self.loan_to_value_bps <= utils::MAX_LTV_BPS);
        let amounts_ok = self.max_loan_amount >= self.min_invoice_amount;
        let days_ok = self.max_invoice_age_days <= utils::MAX_INVOICE_AGE_DAYS;
        let time_ok = self.current_time > 0;
        ltv_ok & amounts_ok & days_ok & time_ok
    }

    /// Enforces all lending rules on an invoice and returns the computed loan amount.
    /// Rules enforced:
    /// 1. Invoice fields are valid (amount, currency, dates, age)
    /// 2. Amount is within the lending range
    /// 3. LTV ratio is respected
    pub fn enforce(self, invoice: InvoiceData) -> u64 {
        let invoice_valid = invoice.validate(self.current_time, self.max_invoice_age_days);
        assert(invoice_valid);

        let amount = invoice.get_amount();
        assert(amount >= self.min_invoice_amount);
        assert(amount <= self.max_loan_amount);

        let loan = utils::compute_loan_amount(amount, self.loan_to_value_bps);
        loan
    }
}

#[test]
fn test_lending_params_valid() {
    let params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };
    assert(params.validate());
}

#[test]
fn test_lending_params_invalid_ltv_zero() {
    let params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 0,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };
    assert(!params.validate());
}

#[test]
fn test_lending_params_invalid_ltv_above_max() {
    let params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 20_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };
    assert(!params.validate());
}

#[test]
fn test_enforce_computes_correct_loan() {
    let params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 8_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 1_000_000,  // $10,000.00 in cents
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let loan = params.enforce(invoice);
    // 80% of $10,000 = $8,000 = 800,000 cents
    assert(loan == 800_000);
}

#[test]
fn test_enforce_ltv_60_percent() {
    let params = LendingParams {
        max_loan_amount: 100_000_000,
        loan_to_value_bps: 6_000,
        min_invoice_amount: 100_000,
        max_invoice_age_days: 90,
        current_time: 1_719_000_000,
    };
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 1_000_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let loan = params.enforce(invoice);
    // 60% of $10,000 = $6,000 = 600,000 cents
    assert(loan == 600_000);
}