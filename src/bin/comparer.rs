use clap::Parser;
use ypbank::{CsvFormat, bin_format::BinFormat, txt_format::TxtFormat, compare, CompareResult};
use ypbank::error::BankFormatError;
use std::fs::File;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    file1: String,
    #[arg(long)]
    format1: String,
    #[arg(long)]
    file2: String,
    #[arg(long)]
    format2: String,
}

fn main() -> Result<(), BankFormatError> {
    let args = Args::parse();

    let mut f1 = File::open(&args.file1)?;
    let mut f2 = File::open(&args.file2)?;

let result = match (args.format1.as_str(), args.format2.as_str()) {
    ("binary", "csv")    => compare::<BinFormat, CsvFormat>(&mut f1, &mut f2)?,
    ("binary", "txt")    => compare::<BinFormat, TxtFormat>(&mut f1, &mut f2)?,
    ("binary", "binary") => compare::<BinFormat, BinFormat>(&mut f1, &mut f2)?,
    ("csv", "binary")    => compare::<CsvFormat, BinFormat>(&mut f1, &mut f2)?,
    ("csv", "txt")       => compare::<CsvFormat, TxtFormat>(&mut f1, &mut f2)?,
    ("csv", "csv")       => compare::<CsvFormat, CsvFormat>(&mut f1, &mut f2)?,
    ("txt", "binary")    => compare::<TxtFormat, BinFormat>(&mut f1, &mut f2)?,
    ("txt", "csv")       => compare::<TxtFormat, CsvFormat>(&mut f1, &mut f2)?,
    ("txt", "txt")       => compare::<TxtFormat, TxtFormat>(&mut f1, &mut f2)?,
    _ => return Err(BankFormatError::Parse("unsupported format combination".to_string())),
};

    match result {
        CompareResult::Identical => println!(
            "The transaction records in '{}' and '{}' are identical.",
            args.file1, args.file2
        ),
        CompareResult::Mismatch { missing_in_1, missing_in_2 } => {
            for id in missing_in_1 {
                println!("Transaction {} is missing in '{}'", id, args.file1);
            }
            for id in missing_in_2 {
                println!("Transaction {} is missing in '{}'", id, args.file2);
            }
        }
    }

    Ok(())
}