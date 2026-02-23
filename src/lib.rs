//! # ypbank
//!
//! A library for parsing serializing and comparing bank transaction records
//! in multiple formats: CSV, binary, and plain text.
pub mod bin_format;
pub mod csv_format;
pub mod error;
pub mod txt_format;
use std::collections::HashMap;
use std::fmt;

pub use csv_format::CsvFormat;
use error::BankFormatError;

/// Unique transaction identifier type.
pub type TxId = u64;

/// Represents a single bank transaction.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    /// Unique transaction identifier.
    pub tx_id: TxId,
    /// Type of the transaction.
    pub tx_type: TxType,
    /// Sender user ID. For system deposits (`DEPOSIT`), this is `0`.
    pub from_user_id: i64,
    /// Recipient user ID. For system withdrawals (`WITHDRAWAL`), this is `0`.
    pub to_user_id: i64,
    /// Transaction amount in smallest currency units (e.g. cents).
    pub amount: i64,
    /// Unix timestamp in milliseconds since epoch.
    pub timestamp: i64,
    /// Current status of the transaction.
    pub status: Status,
    /// Human-readable description of the transaction.
    pub description: String,
}

impl fmt::Display for TxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxType::Deposit => write!(f, "DEPOSIT"),
            TxType::Transfer => write!(f, "TRANSFER"),
            TxType::Withdrawal => write!(f, "WITHDRAWAL"),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Success => write!(f, "SUCCESS"),
            Status::Failure => write!(f, "FAILURE"),
            Status::Pending => write!(f, "PENDING"),
        }
    }
}

/// The type of a bank transaction.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TxType {
    /// Funds deposited into the system.
    Deposit,
    /// Funds transferred between two users.
    Transfer,
    /// Funds withdrawn from the system.
    Withdrawal,
}

/// The status of a bank transaction.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Status {
    /// Transaction completed successfully.
    Success,
    /// Transaction failed.
    Failure,
    /// Transaction is pending processing.
    Pending,
}

/// A trait for reading and writing transaction records in a specific format.
///
/// Implement this trait to add support for a new format.
/// Uses [`std::io::Read`] and [`std::io::Write`]
/// works with files, stdin, in-memory buffers, or any other IO source.
pub trait BankFormat: Sized {
    /// Read all transactions from the given reader.
    fn read_all<R: std::io::Read>(r: &mut R) -> Result<Vec<Transaction>, BankFormatError>;
    /// Write all transactions to the given writer.
    fn write_all<W: std::io::Write>(
        w: &mut W,
        records: &[Transaction],
    ) -> Result<(), BankFormatError>;
}

/// Convert transaction records from one format to another.
///
/// Reads from `r` using format `From` and writes to `w` using format `To`.
///
/// # Example
/// ```no_run
/// convert::<CsvFormat, BinFormat>(&mut input, &mut output)?;
/// ```
pub fn convert<From, To>(
    r: &mut impl std::io::Read,
    w: &mut impl std::io::Write,
) -> Result<(), BankFormatError>
where
    From: BankFormat,
    To: BankFormat,
{
    let transactions = From::read_all(r)?;
    To::write_all(w, &transactions)
}

/// Compare transaction records from two readers, potentially in different formats.
///
/// Returns [`CompareResult::Identical`] if both sources contain the same transactions
/// (matched by [`TxId`]), or [`CompareResult::Mismatch`] listing missing IDs from each side.
pub fn compare<F1, F2>(
    r1: &mut impl std::io::Read,
    r2: &mut impl std::io::Read,
) -> Result<CompareResult, BankFormatError>
where
    F1: BankFormat,
    F2: BankFormat,
{
    let transactions_one = F1::read_all(r1)?;
    let transactions_two = F2::read_all(r2)?;

    let map1: HashMap<TxId, Transaction> =
        transactions_one.into_iter().map(|t| (t.tx_id, t)).collect();
    let map2: HashMap<TxId, Transaction> =
        transactions_two.into_iter().map(|t| (t.tx_id, t)).collect();

    let mut missing_in_2 = vec![];
    let mut missing_in_1 = vec![];

    for id in map1.keys() {
        if !map2.contains_key(id) {
            missing_in_2.push(*id);
        }
    }
    for id in map2.keys() {
        if !map1.contains_key(id) {
            missing_in_1.push(*id);
        }
    }

    if missing_in_1.is_empty() && missing_in_2.is_empty() {
        Ok(CompareResult::Identical)
    } else {
        Ok(CompareResult::Mismatch {
            missing_in_1,
            missing_in_2,
        })
    }
}

/// The result of comparing two sets of transaction records.
pub enum CompareResult {
    /// Both sources contain identical transaction records.
    Identical,
    /// The sources differ. Each field lists transaction IDs missing from that source.
    Mismatch {
        /// Transaction IDs present in source 2 but missing in source 1.
        missing_in_1: Vec<TxId>,
        /// Transaction IDs present in source 1 but missing in source 2.
        missing_in_2: Vec<TxId>,
    },
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
