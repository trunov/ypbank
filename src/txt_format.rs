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
            writeln!(w, "# Record {} ({:?})", i + 1, tx.tx_type).map_err(BankFormatError::Io)?;
            writeln!(w, "TX_ID: {}", tx.tx_id).map_err(BankFormatError::Io)?;
            writeln!(w, "TX_TYPE: {:?}", tx.tx_type).map_err(BankFormatError::Io)?;
            writeln!(w, "FROM_USER_ID: {}", tx.from_user_id).map_err(BankFormatError::Io)?;
            writeln!(w, "TO_USER_ID: {}", tx.to_user_id).map_err(BankFormatError::Io)?;
            writeln!(w, "AMOUNT: {}", tx.amount).map_err(BankFormatError::Io)?;
            writeln!(w, "TIMESTAMP: {}", tx.timestamp).map_err(BankFormatError::Io)?;
            writeln!(w, "STATUS: {:?}", tx.status).map_err(BankFormatError::Io)?;
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
