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
                format!("{:?}", tx.tx_type),
                tx.from_user_id.to_string(),
                tx.to_user_id.to_string(),
                tx.amount.to_string(),
                tx.timestamp.to_string(),
                format!("{:?}", tx.status),
                tx.description.clone(),
            ])
            .map_err(|e| BankFormatError::Csv(e))?;
        }

        wtr.flush().map_err(|e| BankFormatError::Io(e))?;
        Ok(())
    }
}
