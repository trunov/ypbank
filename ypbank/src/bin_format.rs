use crate::error::BankFormatError;
use crate::{BankFormat, Status, Transaction, TxId, TxType};
use std::io::{Read, Write};

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E]; // 'YPBN'
const MAX_DESC_LEN: usize = 4096;

pub struct BinFormat;

impl BankFormat for BinFormat {
    fn read_all<R: Read>(r: &mut R) -> Result<Vec<Transaction>, BankFormatError> {
        let mut transactions = Vec::new();

        loop {
            let mut magic = [0u8; 4];
            match r.read_exact(&mut magic) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(BankFormatError::Io(e)),
            }

            if magic != MAGIC {
                return Err(BankFormatError::InvalidBinary(format!(
                    "invalid magic: {:?}",
                    magic
                )));
            }

            // read record size
            let mut buf4 = [0u8; 4];
            r.read_exact(&mut buf4).map_err(BankFormatError::Io)?;
            let record_size = u32::from_be_bytes(buf4);
            if record_size < 46 {
                return Err(BankFormatError::InvalidBinary(format!(
                    "record_size {} is too small, minimum is 46 bytes",
                    record_size
                )));
            }

            // TX_ID
            let mut buf8 = [0u8; 8];
            r.read_exact(&mut buf8).map_err(BankFormatError::Io)?;
            let tx_id = u64::from_be_bytes(buf8) as TxId;

            // TX_TYPE
            let mut buf1 = [0u8; 1];
            r.read_exact(&mut buf1).map_err(BankFormatError::Io)?;
            let tx_type = match buf1[0] {
                0 => TxType::Deposit,
                1 => TxType::Transfer,
                2 => TxType::Withdrawal,
                other => {
                    return Err(BankFormatError::InvalidBinary(format!(
                        "unknown tx_type byte: {}",
                        other
                    )));
                }
            };

            // FROM_USER_ID
            r.read_exact(&mut buf8).map_err(BankFormatError::Io)?;
            let from_user_id = u64::from_be_bytes(buf8) as i64;

            // TO_USER_ID
            r.read_exact(&mut buf8).map_err(BankFormatError::Io)?;
            let to_user_id = u64::from_be_bytes(buf8) as i64;

            // AMOUNT
            r.read_exact(&mut buf8).map_err(BankFormatError::Io)?;
            let amount = i64::from_be_bytes(buf8);

            // TIMESTAMP
            r.read_exact(&mut buf8).map_err(BankFormatError::Io)?;
            let timestamp = u64::from_be_bytes(buf8) as i64;

            // STATUS
            r.read_exact(&mut buf1).map_err(BankFormatError::Io)?;
            let status = match buf1[0] {
                0 => Status::Success,
                1 => Status::Failure,
                2 => Status::Pending,
                other => {
                    return Err(BankFormatError::InvalidBinary(format!(
                        "unknown status byte: {}",
                        other
                    )));
                }
            };

            // DESC_LEN
            r.read_exact(&mut buf4).map_err(BankFormatError::Io)?;
            let desc_len = u32::from_be_bytes(buf4) as usize;
            if desc_len > MAX_DESC_LEN {
                return Err(BankFormatError::InvalidBinary(format!(
                    "description length {} exceeds maximum allowed {}",
                    desc_len, MAX_DESC_LEN
                )));
            }

            // DESCRIPTION
            let description = if desc_len > 0 {
                let mut desc_buf = vec![0u8; desc_len];
                r.read_exact(&mut desc_buf).map_err(BankFormatError::Io)?;
                String::from_utf8(desc_buf)
                    .map_err(|e| BankFormatError::InvalidBinary(e.to_string()))?
            } else {
                String::new()
            };

            transactions.push(Transaction {
                tx_id,
                tx_type,
                from_user_id,
                to_user_id,
                amount,
                timestamp,
                status,
                description,
            });
        }

        Ok(transactions)
    }

    fn write_all<W: Write>(w: &mut W, records: &[Transaction]) -> Result<(), BankFormatError> {
        for tx in records {
            let desc_bytes = tx.description.as_bytes();
            let desc_len = desc_bytes.len() as u32;

            // body size: 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4 + desc_len
            let record_size: u32 = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4 + desc_len;

            // magic
            w.write_all(&MAGIC).map_err(BankFormatError::Io)?;

            // record size
            w.write_all(&record_size.to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // TX_ID
            w.write_all(&(tx.tx_id as TxId).to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // TX_TYPE
            let tx_type_byte: u8 = match tx.tx_type {
                TxType::Deposit => 0,
                TxType::Transfer => 1,
                TxType::Withdrawal => 2,
            };
            w.write_all(&[tx_type_byte]).map_err(BankFormatError::Io)?;

            // FROM_USER_ID
            w.write_all(&(tx.from_user_id as u64).to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // TO_USER_ID
            w.write_all(&(tx.to_user_id as u64).to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // AMOUNT
            w.write_all(&tx.amount.to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // TIMESTAMP
            w.write_all(&(tx.timestamp as u64).to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // STATUS
            let status_byte: u8 = match tx.status {
                Status::Success => 0,
                Status::Failure => 1,
                Status::Pending => 2,
            };
            w.write_all(&[status_byte]).map_err(BankFormatError::Io)?;

            // DESC_LEN
            w.write_all(&desc_len.to_be_bytes())
                .map_err(BankFormatError::Io)?;

            // DESCRIPTION
            if desc_len > 0 {
                w.write_all(desc_bytes).map_err(BankFormatError::Io)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn valid_transaction() -> Transaction {
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

    fn make_valid_record() -> Vec<u8> {
        let mut buf = Vec::new();
        BinFormat::write_all(&mut buf, &[valid_transaction()]).unwrap();
        buf
    }

    #[test]
    fn test_invalid_binary_cases() {
        let full = make_valid_record();

        let cases: Vec<(Vec<u8>, &str)> = vec![
            // invalid magic
            (vec![0x00, 0x00, 0x00, 0x00], "invalid magic"),
            // invalid tx_type
            (
                {
                    let mut b = full.clone();
                    b[16] = 0x04;
                    b
                },
                "unknown tx_type byte: 4",
            ),
            // invalid status
            (
                {
                    let mut b = full.clone();
                    b[49] = 0x04;
                    b
                },
                "unknown status byte: 4",
            ),
        ];

        for (bad_data, expected_msg) in cases {
            let mut cursor = Cursor::new(bad_data);
            match BinFormat::read_all(&mut cursor) {
                Err(BankFormatError::InvalidBinary(msg)) => {
                    assert!(msg.contains(expected_msg), "got: {}", msg);
                }
                other => panic!("expected InvalidBinary, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_read_all_valid_transaction() {
        let buf = make_valid_record();
        let mut cursor = Cursor::new(buf);

        match BinFormat::read_all(&mut cursor) {
            Ok(transactions) => {
                assert_eq!(transactions.len(), 1);
                assert_eq!(transactions[0], valid_transaction());
            }
            Err(e) => panic!("expected Ok, got error: {}", e),
        }
    }

    #[test]
    fn test_bin_roundtrip() -> Result<(), BankFormatError> {
        let original = vec![Transaction {
            tx_id: 1,
            tx_type: TxType::Deposit,
            from_user_id: 10,
            to_user_id: 20,
            amount: 1000,
            timestamp: 1234567890,
            status: Status::Success,
            description: "food".to_string(),
        }];

        let mut buf = Vec::new();
        BinFormat::write_all(&mut buf, &original)?;

        let mut cursor = Cursor::new(buf);
        let parsed = BinFormat::read_all(&mut cursor)?;

        assert_eq!(original, parsed);
        Ok(())
    }
}
