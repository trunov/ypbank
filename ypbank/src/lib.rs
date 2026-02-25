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
///  // convert::<CsvFormat, BinFormat>(&mut input, &mut output)?;
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
    let mut differing = vec![];

    for (id, tx1) in &map1 {
        match map2.get(id) {
            None => missing_in_2.push(*id),
            Some(tx2) if tx1 != tx2 => differing.push((*id, tx1.clone(), tx2.clone())),
            _ => {}
        }
    }

    for id in map2.keys() {
        if !map1.contains_key(id) {
            missing_in_1.push(*id);
        }
    }

    if missing_in_1.is_empty() && missing_in_2.is_empty() && differing.is_empty() {
        Ok(CompareResult::Identical)
    } else {
        Ok(CompareResult::Mismatch {
            missing_in_1,
            missing_in_2,
            differing,
        })
    }
}

/// The result of comparing two sets of transaction records.
#[derive(Debug)]
pub enum CompareResult {
    /// Both sources contain identical transaction records.
    Identical,
    /// The sources differ. Each field lists transaction IDs missing from that source.
    Mismatch {
        /// Transaction IDs present in source 2 but missing in source 1.
        missing_in_1: Vec<TxId>,
        /// Transaction IDs present in source 1 but missing in source 2.
        missing_in_2: Vec<TxId>,
        //// Transactions present in both sources but with differing fields.
        differing: Vec<(TxId, Transaction, Transaction)>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin_format::BinFormat;
    use crate::csv_format::CsvFormat;
    use crate::txt_format::TxtFormat;
    use std::io::Cursor;

    fn expected_transaction() -> Transaction {
        Transaction {
            tx_id: 1,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 42,
            amount: 1000,
            timestamp: 1234567890,
            status: Status::Success,
            description: "test".to_string(),
        }
    }

    // --- convert tests ---

    #[test]
    fn test_convert_csv_to_bin() {
        let csv = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                   1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";

        let mut input = Cursor::new(csv);
        let mut output = Vec::new();

        convert::<CsvFormat, BinFormat>(&mut input, &mut output).unwrap();

        let mut cursor = Cursor::new(output);
        let transactions = BinFormat::read_all(&mut cursor).unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0], expected_transaction());
    }

    #[test]
    fn test_convert_bin_to_csv() {
        let original = vec![expected_transaction()];
        let mut bin_buf = Vec::new();
        BinFormat::write_all(&mut bin_buf, &original).unwrap();

        let mut input = Cursor::new(bin_buf);
        let mut output = Vec::new();

        convert::<BinFormat, CsvFormat>(&mut input, &mut output).unwrap();

        let mut cursor = Cursor::new(output);
        let transactions = CsvFormat::read_all(&mut cursor).unwrap();
        assert_eq!(transactions, original);
    }

    #[test]
    fn test_convert_csv_to_txt() {
        let csv = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                   1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";

        let mut input = Cursor::new(csv);
        let mut output = Vec::new();

        convert::<CsvFormat, TxtFormat>(&mut input, &mut output).unwrap();

        let mut cursor = Cursor::new(output);
        let transactions = TxtFormat::read_all(&mut cursor).unwrap();
        assert_eq!(transactions[0], expected_transaction());
    }

    // --- compare tests ---

    #[test]
    fn test_compare_identical_same_format() {
        let csv = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                   1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";

        let mut r1 = Cursor::new(csv);
        let mut r2 = Cursor::new(csv);

        match compare::<CsvFormat, CsvFormat>(&mut r1, &mut r2).unwrap() {
            CompareResult::Identical => {}
            other => panic!("expected Identical, got {:?}", other),
        }
    }

    #[test]
    fn test_compare_identical_different_formats() {
        let original = vec![expected_transaction()];
        let mut bin_buf = Vec::new();
        BinFormat::write_all(&mut bin_buf, &original).unwrap();

        let csv = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                   1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";

        let mut r1 = Cursor::new(bin_buf);
        let mut r2 = Cursor::new(csv);

        assert!(matches!(
            compare::<BinFormat, CsvFormat>(&mut r1, &mut r2).unwrap(),
            CompareResult::Identical
        ));
    }

    #[test]
    fn test_compare_missing_in_second() {
        let csv1 = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n\
                2,TRANSFER,10,20,500,1234567891,PENDING,second\n";
        let csv2 = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";
        let mut r1 = Cursor::new(csv1);
        let mut r2 = Cursor::new(csv2);
        match compare::<CsvFormat, CsvFormat>(&mut r1, &mut r2).unwrap() {
            CompareResult::Mismatch {
                missing_in_1,
                missing_in_2,
                differing,
            } => {
                assert!(missing_in_1.is_empty());
                assert_eq!(missing_in_2, vec![2]);
                assert!(differing.is_empty());
            }
            _ => panic!("expected Mismatch"),
        }
    }

    #[test]
    fn test_compare_missing_in_first() {
        let csv1 = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";
        let csv2 = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n\
                2,TRANSFER,10,20,500,1234567891,PENDING,second\n";
        let mut r1 = Cursor::new(csv1);
        let mut r2 = Cursor::new(csv2);
        match compare::<CsvFormat, CsvFormat>(&mut r1, &mut r2).unwrap() {
            CompareResult::Mismatch {
                missing_in_1,
                missing_in_2,
                differing,
            } => {
                assert_eq!(missing_in_1, vec![2]);
                assert!(missing_in_2.is_empty());
                assert!(differing.is_empty());
            }
            _ => panic!("expected Mismatch"),
        }
    }

    #[test]
    fn test_compare_differing_fields() {
        let csv1 = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n";
        let csv2 = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                1,DEPOSIT,0,42,9999,1234567890,SUCCESS,test\n"; // amount differs
        let mut r1 = Cursor::new(csv1);
        let mut r2 = Cursor::new(csv2);
        match compare::<CsvFormat, CsvFormat>(&mut r1, &mut r2).unwrap() {
            CompareResult::Mismatch {
                missing_in_1,
                missing_in_2,
                differing,
            } => {
                assert!(missing_in_1.is_empty());
                assert!(missing_in_2.is_empty());
                assert_eq!(differing.len(), 1);
                assert_eq!(differing[0].0, 1);
            }
            _ => panic!("expected Mismatch"),
        }
    }
}
