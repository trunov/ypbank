use std::fs::File;
use std::io;

use ypbank::{BankFormat, CsvFormat, txt_format::TxtFormat};

pub fn read_file(path: &str) -> io::Result<File> {
    let file = File::open(path)?;
    Ok(file)
}

fn main() {
    // read
    let mut file = match read_file("tx.csv") {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open file: {e}");
            return;
        }
    };

    let transactions = match CsvFormat::read_all(&mut file) {
        Ok(t) => t,
        Err(e) => {
            println!("Parse error: {e}");
            return;
        }
    };

    for tx in &transactions {
        println!("{:?}", tx);
    }

    let mut file = match read_file("tx.txt") {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open file: {e}");
            return;
        }
    };

    let txt_transactions = match TxtFormat::read_all(&mut file) {
        Ok(t) => t,
        Err(e) => {
            println!("Parse error: {e}");
            return;
        }
    };

    for tx in &txt_transactions {
        println!("{:?}", tx);
    }
}
