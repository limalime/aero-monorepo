use soroban_sdk::{contracterror, contracttype, Address, BytesN, Env};

// --- Asset Types ---

/// Identifies which asset is being used for a deposit, loan, or bond.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asset {
    /// Native XLM (Stellar lumens)
    XLM,
    /// USDC via Stellar Asset Contract (SAC)
    USDC,
}

// --- Loan Status ---

/// Tracks the lifecycle state of a loan.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoanStatus {
    /// Loan is active and accruing interest
    Active,
    /// Borrower has fully repaid principal + interest
    Repaid,
    /// Loan has been liquidated due to default
    Defaulted,
}

// --- Core Data Structures ---

/// Represents a single loan on the protocol.
#[contracttype]
#[derive(Clone, Debug)]
pub struct LoanData {
    /// Borrower address
    pub borrower: Address,
    /// Invoice commitment hash from the ZK circuit
    pub invoice_hash: BytesN<32>,
    /// Principal amount requested (in stroops or USDC cents)
    pub loan_amount: i128,
    /// Asset used for this loan
    pub asset: Asset,
    /// 10% performance bond posted by borrower
    pub skin_in_game: i128,
    /// Loan-to-value ratio in basis points
    pub ltv_bps: u32,
    /// Interest rate in basis points per annum
    pub interest_bps: u32,
    /// Current loan status
    pub status: LoanStatus,
    /// Timestamp when the loan was created
    pub created_at: u64,
    /// Timestamp when the loan matures
    pub due_date: u64,
    /// Total principal repaid so far
    pub amount_repaid: i128,
    /// Total interest locked at first repayment (prevents recalculation drift)
    pub accrued_interest: i128,
}

/// Tracks a lender's position in the liquidity pool for one asset.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolPosition {
    /// Original amount deposited (immutable after deposit; never decreased on withdrawal)
    pub deposit_amount: i128,
    /// Pool shares held (proportional to deposit)
    pub shares: i128,
}

/// Pool statistics for one asset type.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolStats {
    /// Total underlying asset in the pool
    pub total_deposits: i128,
    /// Total shares issued
    pub total_shares: i128,
    /// Total amount currently lent out
    pub total_lent: i128,
    /// Available liquidity (deposits - lent)
    pub available_liquidity: i128,
}

// --- Storage Keys ---

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Contract administrator
    Admin,
    /// Address of the USDC Stellar Asset Contract
    UsdcAddress,
    /// Loan counter (next loan ID)
    LoanCounter,
    /// Loan data for a given loan ID
    Loan(u64),
    /// Pool stats for a given asset
    PoolStats(Asset),
    /// Lender's pool position for a given asset
    PoolPosition(Address, Asset),
    /// Whether the contract has been initialized
    Initialized,
    /// Nullifier registry for double-spend prevention
    NullifierUsed(BytesN<32>),
}

// --- Contract Errors ---

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InsufficientBalance = 5,
    InsufficientPoolLiquidity = 6,
    LoanNotFound = 7,
    LoanNotActive = 8,
    LoanExpired = 9,
    InvalidAsset = 10,
    InvalidSkinInGame = 11,
    ProofVerificationFailed = 12,
    RepaymentTooSmall = 13,
    AlreadyRepaid = 14,
    NotBorrower = 15,
    TransferFailed = 16,
    Overflow = 17,
    LoanNotDue = 18,
    NullifierAlreadyUsed = 19,
}

// --- Protocol Constants ---

/// Required skin-in-the-game: 10% of loan amount (1000 basis points).
pub const SKIN_IN_GAME_BPS: i128 = 1000;
/// One XLM in stroops.
pub const ONE_XLM: i128 = 10_000_000;
/// Well-known native XLM Stellar Asset Contract address.
pub const NATIVE_XLM_ADDRESS: &str = "CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUF34TQ6NPVHSZ2IXIZQNQAL";

/// Returns the native XLM Stellar Asset Contract address for the given environment.
pub fn get_native_xlm_address(env: &Env) -> Address {
    Address::from_string(&soroban_sdk::String::from_str(env, NATIVE_XLM_ADDRESS))
}