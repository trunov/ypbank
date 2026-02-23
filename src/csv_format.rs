use crate::error::BankFormatError;
use crate::{BankFormat, Status, Transaction, TxType};

pub struct CsvFormat;

impl BankFormat for CsvFormat {
    fn read_all<R: std::io::Read>(r: &mut R) -> Result<Vec<Transaction>, BankFormatError> {
        let mut rdr = csv::Reader::from_reader(r);
        let mut transactions = Vec::new();

        for result in rdr.records() {
            let record = result.map_err(|e| BankFormatError::Parse(e.to_string()))?;
            transactions.push(Transaction {
                tx_id: record[0]
                    .parse()
                    .map_err(|_| BankFormatError::Parse("tx_id".into()))?,
                tx_type: match &record[1] {
                    "DEPOSIT" => TxType::Deposit,
                    "TRANSFER" => TxType::Transfer,
                    "WITHDRAWAL" => TxType::Withdrawal,
                    other => {
                        return Err(BankFormatError::Parse(format!("unknown tx_type: {other}")));
                    }
                },
                from_user_id: record[2]
                    .parse()
                    .map_err(|_| BankFormatError::Parse("from_user_id".into()))?,
                to_user_id: record[3]
                    .parse()
                    .map_err(|_| BankFormatError::Parse("to_user_id".into()))?,
                amount: record[4]
                    .parse()
                    .map_err(|_| BankFormatError::Parse("amount".into()))?,
                timestamp: record[5]
                    .parse()
                    .map_err(|_| BankFormatError::Parse("timestamp".into()))?,
                status: match &record[6] {
                    "SUCCESS" => Status::Success,
                    "FAILURE" => Status::Failure,
                    "PENDING" => Status::Pending,
                    other => {
                        return Err(BankFormatError::Parse(format!("unknown status: {other}")));
                    }
                },
                description: record[7].to_string(),
            });
        }

        Ok(transactions)
    }

    fn write_all<W: std::io::Write>(
        w: &mut W,
        records: &[Transaction],
    ) -> Result<(), BankFormatError> {
        let mut wtr = csv::Writer::from_writer(w);
        wtr.write_record([
            "tx_id",
            "tx_type",
            "from_user_id",
            "to_user_id",
            "amount",
            "timestamp",
            "status",
            "description",
        ])
        .map_err(|e| BankFormatError::Csv(e))?;

        for tx in records {
            wtr.write_record(&[
                tx.tx_id.to_string(),
                tx.tx_type.to_string(),
                tx.from_user_id.to_string(),
                tx.to_user_id.to_string(),
                tx.amount.to_string(),
                tx.timestamp.to_string(),
                tx.status.to_string(),
                tx.description.clone(),
            ])
            .map_err(|e| BankFormatError::Csv(e))?;
        }

        wtr.flush().map_err(|e| BankFormatError::Io(e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn make_valid_csv() -> String {
        "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
         1,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n"
            .to_string()
    }

    #[test]
    fn test_read_all_valid_record() {
        let mut cursor = Cursor::new(make_valid_csv());
        match CsvFormat::read_all(&mut cursor) {
            Ok(transactions) => {
                assert_eq!(transactions.len(), 1);
                assert_eq!(transactions[0], expected_transaction());
            }
            Err(e) => panic!("expected Ok, got error: {}", e),
        }
    }

    #[test]
    fn test_roundtrip() {
        let original = vec![expected_transaction()];
        let mut buf = Vec::new();
        CsvFormat::write_all(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        match CsvFormat::read_all(&mut cursor) {
            Ok(transactions) => assert_eq!(transactions, original),
            Err(e) => panic!("expected Ok, got error: {}", e),
        }
    }

    #[test]
    fn test_read_all_multiple_records() {
        let csv = "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                   1,DEPOSIT,0,42,1000,1234567890,SUCCESS,first\n\
                   2,TRANSFER,10,20,500,1234567891,PENDING,second\n\
                   3,WITHDRAWAL,99,0,250,1234567892,FAILURE,third\n";

        let mut cursor = Cursor::new(csv);
        match CsvFormat::read_all(&mut cursor) {
            Ok(transactions) => assert_eq!(transactions.len(), 3),
            Err(e) => panic!("expected Ok, got error: {}", e),
        }
    }

    #[test]
    fn test_invalid_parse_cases() {
        let cases: Vec<(&str, &str)> = vec![
            // invalid tx_type
            (
                "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                 1,INVALID,0,42,1000,1234567890,SUCCESS,test\n",
                "unknown tx_type",
            ),
            // invalid status
            (
                "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                 1,DEPOSIT,0,42,1000,1234567890,INVALID,test\n",
                "unknown status",
            ),
            // invalid tx_id
            (
                "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                 abc,DEPOSIT,0,42,1000,1234567890,SUCCESS,test\n",
                "tx_id",
            ),
            // invalid amount
            (
                "tx_id,tx_type,from_user_id,to_user_id,amount,timestamp,status,description\n\
                 1,DEPOSIT,0,42,notanumber,1234567890,SUCCESS,test\n",
                "amount",
            ),
        ];

        for (bad_csv, expected_msg) in cases {
            let mut cursor = Cursor::new(bad_csv);
            match CsvFormat::read_all(&mut cursor) {
                Err(BankFormatError::Parse(msg)) => {
                    assert!(msg.contains(expected_msg), "got: {}", msg);
                }
                other => panic!("expected Parse error, got {:?}", other),
            }
        }
    }
}
