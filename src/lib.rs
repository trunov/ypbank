pub mod csv_format;
pub mod error;
pub mod txt_format;
pub use csv_format::CsvFormat;
use error::BankFormatError;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Transaction {
    pub tx_id: i64,
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
