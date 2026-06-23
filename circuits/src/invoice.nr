use crate::utils;

// Number of fields used in the invoice hash
global INVOICE_HASH_FIELDS: u32 = 8;

/// Represents a real-world invoice with all sensitive fields kept private.
/// All monetary values are in cents (or smallest currency unit).
pub struct InvoiceData {
    pub invoice_id: [u8; 32],
    pub amount: u64,
    pub debtor_hash: [u8; 32],
    pub issuance_date: u64,
    pub due_date: u64,
    pub currency: u8,
}

impl InvoiceData {
    /// Validates that all invoice fields are sane.
    /// Enforces: valid amount range, valid currency, chronological dates,
    /// non-expired, and within age limits.
    pub fn validate(self, current_time: u64, max_age_days: u32) -> bool {
        let amount_ok = utils::is_valid_amount(self.amount);
        let currency_ok = utils::is_valid_currency(self.currency);
        let chrono_ok = utils::is_chronological(self.issuance_date, self.due_date);
        let not_expired = utils::is_not_expired(self.due_date, current_time);
        let age_ok = utils::is_within_age_limit(self.issuance_date, current_time, max_age_days);

        amount_ok & currency_ok & chrono_ok & not_expired & age_ok
    }

    /// Computes a Pedersen commitment over all invoice fields.
    /// This serves as the public on-chain reference without revealing data.
    pub fn commitment(self) -> Field {
        let mut fields = [0; INVOICE_HASH_FIELDS];
        fields[0] = utils::bytes32_to_field(self.invoice_id);
        fields[1] = self.amount as Field;
        fields[2] = utils::bytes32_to_field(self.debtor_hash);
        fields[3] = self.issuance_date as Field;
        fields[4] = self.due_date as Field;
        fields[5] = self.currency as Field;
        // Domain separation: version tag prevents hash collisions across protocol upgrades
        fields[6] = 1;  // Aero Protocol v1
        fields[7] = 0;  // Reserved
        utils::hash_fields(fields)
    }

    /// Returns the invoice amount.
    pub fn get_amount(self) -> u64 {
        self.amount
    }

    /// Returns the invoice ID bytes.
    pub fn get_id(self) -> [u8; 32] {
        self.invoice_id
    }
}

#[test]
fn test_validate_valid_invoice() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 500_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let valid = invoice.validate(1_719_000_000, 90);
    assert(valid);
}

#[test]
fn test_validate_expired_invoice() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 500_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let valid = invoice.validate(1_730_000_000, 90);
    assert(!valid);
}

#[test]
fn test_validate_invalid_amount() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 1,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let valid = invoice.validate(1_719_000_000, 90);
    assert(!valid);
}

#[test]
fn test_validate_invalid_currency() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 500_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: 99,
    };
    let valid = invoice.validate(1_719_000_000, 90);
    assert(!valid);
}

#[test]
fn test_validate_non_chronological() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 500_000,
        debtor_hash: [2; 32],
        issuance_date: 1_721_174_400,
        due_date: 1_718_400_000,
        currency: utils::CURRENCY_USD,
    };
    let valid = invoice.validate(1_719_000_000, 90);
    assert(!valid);
}

#[test]
fn test_commitment_deterministic() {
    let invoice = InvoiceData {
        invoice_id: [1; 32],
        amount: 500_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let c1 = invoice.commitment();
    let c2 = invoice.commitment();
    assert(c1 == c2);
}

#[test]
fn test_commitment_differs_on_amount() {
    let inv1 = InvoiceData {
        invoice_id: [1; 32],
        amount: 500_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    let inv2 = InvoiceData {
        invoice_id: [1; 32],
        amount: 600_000,
        debtor_hash: [2; 32],
        issuance_date: 1_718_400_000,
        due_date: 1_721_174_400,
        currency: utils::CURRENCY_USD,
    };
    assert(inv1.commitment() != inv2.commitment());
}