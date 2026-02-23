use crate::error::BankFormatError;
use crate::{BankFormat, Status, Transaction, TxType};
use std::collections::HashMap;
use std::io::{BufRead, Write};

pub struct TxtFormat;

impl BankFormat for TxtFormat {
    fn read_all<R: std::io::Read>(r: &mut R) -> Result<Vec<Transaction>, BankFormatError> {
        let reader = std::io::BufReader::new(r);
        let mut transactions = Vec::new();
        let mut current: HashMap<String, String> = HashMap::new();

        for line in reader.lines() {
            let line = line.map_err(BankFormatError::Io)?;
            let line = line.trim().to_string();

            if line.starts_with('#') {
                if !current.is_empty() {
                    transactions.push(TxtFormat::parse_map(&current)?);
                    current.clear();
                }
            } else if let Some((key, value)) = line.split_once(':') {
                current.insert(
                    key.trim().to_string(),
                    value.trim().trim_matches('"').to_string(),
                );
            }
        }

        if !current.is_empty() {
            transactions.push(TxtFormat::parse_map(&current)?);
        }

        Ok(transactions)
    }

    fn write_all<W: Write>(w: &mut W, records: &[Transaction]) -> Result<(), BankFormatError> {
        for (i, tx) in records.iter().enumerate() {
            writeln!(w, "# Record {} ({})", i + 1, tx.tx_type).map_err(BankFormatError::Io)?;
            writeln!(w, "TX_ID: {}", tx.tx_id).map_err(BankFormatError::Io)?;
            writeln!(w, "TX_TYPE: {}", tx.tx_type).map_err(BankFormatError::Io)?;
            writeln!(w, "FROM_USER_ID: {}", tx.from_user_id).map_err(BankFormatError::Io)?;
            writeln!(w, "TO_USER_ID: {}", tx.to_user_id).map_err(BankFormatError::Io)?;
            writeln!(w, "AMOUNT: {}", tx.amount).map_err(BankFormatError::Io)?;
            writeln!(w, "TIMESTAMP: {}", tx.timestamp).map_err(BankFormatError::Io)?;
            writeln!(w, "STATUS: {}", tx.status).map_err(BankFormatError::Io)?;
            writeln!(w, "DESCRIPTION: \"{}\"", tx.description).map_err(BankFormatError::Io)?;
            writeln!(w).map_err(BankFormatError::Io)?;
        }
        Ok(())
    }
}

impl TxtFormat {
    fn parse_map(map: &HashMap<String, String>) -> Result<Transaction, BankFormatError> {
        let get = |key: &str| -> Result<&str, BankFormatError> {
            map.get(key)
                .map(|s| s.as_str())
                .ok_or_else(|| BankFormatError::Parse(format!("missing field: {key}")))
        };

        Ok(Transaction {
            tx_id: get("TX_ID")?
                .parse()
                .map_err(|_| BankFormatError::Parse("TX_ID".into()))?,
            tx_type: TxtFormat::parse_tx_type(get("TX_TYPE")?)?,
            from_user_id: get("FROM_USER_ID")?
                .parse()
                .map_err(|_| BankFormatError::Parse("FROM_USER_ID".into()))?,
            to_user_id: get("TO_USER_ID")?
                .parse()
                .map_err(|_| BankFormatError::Parse("TO_USER_ID".into()))?,
            amount: get("AMOUNT")?
                .parse()
                .map_err(|_| BankFormatError::Parse("AMOUNT".into()))?,
            timestamp: get("TIMESTAMP")?
                .parse()
                .map_err(|_| BankFormatError::Parse("TIMESTAMP".into()))?,
            status: TxtFormat::parse_status(get("STATUS")?)?,
            description: get("DESCRIPTION")?.to_string(),
        })
    }

    fn parse_tx_type(s: &str) -> Result<TxType, BankFormatError> {
        match s {
            "DEPOSIT" => Ok(TxType::Deposit),
            "TRANSFER" => Ok(TxType::Transfer),
            "WITHDRAWAL" => Ok(TxType::Withdrawal),
            other => Err(BankFormatError::Parse(format!("unknown tx_type: {other}"))),
        }
    }

    fn parse_status(s: &str) -> Result<Status, BankFormatError> {
        match s {
            "SUCCESS" => Ok(Status::Success),
            "FAILURE" => Ok(Status::Failure),
            "PENDING" => Ok(Status::Pending),
            other => Err(BankFormatError::Parse(format!("unknown status: {other}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Status, TxType};
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

    fn make_valid_txt() -> String {
        "# Record 1 (DEPOSIT)\n\
         TX_ID: 1\n\
         TX_TYPE: DEPOSIT\n\
         FROM_USER_ID: 0\n\
         TO_USER_ID: 42\n\
         AMOUNT: 1000\n\
         TIMESTAMP: 1234567890\n\
         STATUS: SUCCESS\n\
         DESCRIPTION: \"test\"\n\n"
            .to_string()
    }

    #[test]
    fn test_read_all_valid_record() {
        let mut cursor = Cursor::new(make_valid_txt());
        match TxtFormat::read_all(&mut cursor) {
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
        TxtFormat::write_all(&mut buf, &original).unwrap();

        let mut cursor = Cursor::new(buf);
        match TxtFormat::read_all(&mut cursor) {
            Ok(transactions) => assert_eq!(transactions, original),
            Err(e) => panic!("expected Ok, got error: {}", e),
        }
    }

    #[test]
    fn test_invalid_parse_cases() {
        let cases: Vec<(String, &str)> = vec![
            // invalid tx_type
            (
                "# Record 1\n\
                 TX_ID: 1\n\
                 TX_TYPE: INVALID\n\
                 FROM_USER_ID: 0\n\
                 TO_USER_ID: 42\n\
                 AMOUNT: 1000\n\
                 TIMESTAMP: 1234567890\n\
                 STATUS: SUCCESS\n\
                 DESCRIPTION: \"test\"\n\n"
                    .to_string(),
                "unknown tx_type",
            ),
            // invalid status
            (
                "# Record 1\n\
                 TX_ID: 1\n\
                 TX_TYPE: DEPOSIT\n\
                 FROM_USER_ID: 0\n\
                 TO_USER_ID: 42\n\
                 AMOUNT: 1000\n\
                 TIMESTAMP: 1234567890\n\
                 STATUS: INVALID\n\
                 DESCRIPTION: \"test\"\n\n"
                    .to_string(),
                "unknown status",
            ),
            // missing field
            (
                "# Record 1\n\
                 TX_ID: 1\n\
                 TX_TYPE: DEPOSIT\n\
                 DESCRIPTION: \"test\"\n\n"
                    .to_string(),
                "missing field",
            ),
            // invalid tx_id
            (
                "# Record 1\n\
                 TX_ID: abc\n\
                 TX_TYPE: DEPOSIT\n\
                 FROM_USER_ID: 0\n\
                 TO_USER_ID: 42\n\
                 AMOUNT: 1000\n\
                 TIMESTAMP: 1234567890\n\
                 STATUS: SUCCESS\n\
                 DESCRIPTION: \"test\"\n\n"
                    .to_string(),
                "TX_ID",
            ),
        ];

        for (bad_txt, expected_msg) in cases {
            let mut cursor = Cursor::new(bad_txt);
            match TxtFormat::read_all(&mut cursor) {
                Err(BankFormatError::Parse(msg)) => {
                    assert!(msg.contains(expected_msg), "got: {}", msg);
                }
                other => panic!("expected Parse error, got {:?}", other),
            }
        }
    }
}
