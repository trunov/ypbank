pub mod csv_format;
pub mod bin_format;
pub mod error;
pub mod txt_format;
use std::collections::HashMap;

pub use csv_format::CsvFormat;
use error::BankFormatError;

pub type TxId = u64;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    pub tx_id: TxId,
    pub tx_type: TxType,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub amount: i64,
    pub timestamp: i64,
    pub status: Status,
    pub description: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Status {
    Success,
    Failure,
    Pending,
}

pub trait BankFormat: Sized {
    fn read_all<R: std::io::Read>(r: &mut R) -> Result<Vec<Transaction>, BankFormatError>;
    fn write_all<W: std::io::Write>(
        w: &mut W,
        records: &[Transaction],
    ) -> Result<(), BankFormatError>;
}

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

    let map1: HashMap<TxId, Transaction> = transactions_one.into_iter().map(|t| (t.tx_id, t)).collect();
    let map2: HashMap<TxId, Transaction> = transactions_two.into_iter().map(|t| (t.tx_id, t)).collect();

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
        Ok(CompareResult::Mismatch { missing_in_1, missing_in_2 })
    }
}

pub enum CompareResult {
    Identical,
    Mismatch {
        missing_in_1: Vec<TxId>,
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
