use clap::{Parser, ValueEnum};
use std::fs::File;
use ypbank::error::BankFormatError;
use ypbank::{CompareResult, CsvFormat, bin_format::BinFormat, compare, txt_format::TxtFormat};

#[derive(ValueEnum, Clone)]
enum Format {
    Csv,
    Txt,
    Binary,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    file1: String,
    #[arg(long)]
    format1: Format,
    #[arg(long)]
    file2: String,
    #[arg(long)]
    format2: Format,
}

fn main() -> Result<(), BankFormatError> {
    let args = Args::parse();

    let mut f1 = File::open(&args.file1)?;
    let mut f2 = File::open(&args.file2)?;

    let result = match (args.format1, args.format2) {
        (Format::Binary, Format::Csv) => compare::<BinFormat, CsvFormat>(&mut f1, &mut f2)?,
        (Format::Binary, Format::Txt) => compare::<BinFormat, TxtFormat>(&mut f1, &mut f2)?,
        (Format::Binary, Format::Binary) => compare::<BinFormat, BinFormat>(&mut f1, &mut f2)?,
        (Format::Csv, Format::Binary) => compare::<CsvFormat, BinFormat>(&mut f1, &mut f2)?,
        (Format::Csv, Format::Txt) => compare::<CsvFormat, TxtFormat>(&mut f1, &mut f2)?,
        (Format::Csv, Format::Csv) => compare::<CsvFormat, CsvFormat>(&mut f1, &mut f2)?,
        (Format::Txt, Format::Binary) => compare::<TxtFormat, BinFormat>(&mut f1, &mut f2)?,
        (Format::Txt, Format::Csv) => compare::<TxtFormat, CsvFormat>(&mut f1, &mut f2)?,
        (Format::Txt, Format::Txt) => compare::<TxtFormat, TxtFormat>(&mut f1, &mut f2)?,
    };

    match result {
        CompareResult::Identical => println!(
            "The transaction records in '{}' and '{}' are identical.",
            args.file1, args.file2
        ),
        CompareResult::Mismatch {
            missing_in_1,
            missing_in_2,
            differing,
        } => {
            for id in missing_in_1 {
                println!("Transaction {} is missing in '{}'", id, args.file1);
            }
            for id in missing_in_2 {
                println!("Transaction {} is missing in '{}'", id, args.file2);
            }
            for (id, tx1, tx2) in differing {
                println!(
                    "Transaction {} differs between '{}' and '{}':",
                    id, args.file1, args.file2
                );
                println!("  {}: {:?}", args.file1, tx1);
                println!("  {}: {:?}", args.file2, tx2);
            }
        }
    }

    Ok(())
}
