use clap::{Parser, ValueEnum};
use std::fs::File;
use ypbank::{CsvFormat, txt_format::TxtFormat, bin_format::BinFormat, convert};
use ypbank::error::BankFormatError;

#[derive(Parser)]
#[command(name = "ypbank_converter")]
struct Cli {
    #[arg(long)]
    input: std::path::PathBuf,

    #[arg(long, value_enum)]
    input_format: Format,

    #[arg(long, value_enum)]
    output_format: Format,
}

#[derive(ValueEnum, Clone)]
enum Format {
    Csv,
    Txt,
    Bin,
}

fn main() -> Result<(), BankFormatError> {
    let cli = Cli::parse();
    let mut input = File::open(&cli.input)?;
    let mut stdout = std::io::stdout().lock();
    match (cli.input_format, cli.output_format) {
        (Format::Csv, Format::Txt) => convert::<CsvFormat, TxtFormat>(&mut input, &mut stdout)?,
        (Format::Txt, Format::Csv) => convert::<TxtFormat, CsvFormat>(&mut input, &mut stdout)?,
        (Format::Csv, Format::Bin) => convert::<CsvFormat, BinFormat>(&mut input, &mut stdout)?,
        (Format::Txt, Format::Bin) => convert::<TxtFormat, BinFormat>(&mut input, &mut stdout)?,
        (Format::Bin, Format::Csv) => convert::<BinFormat, CsvFormat>(&mut input, &mut stdout)?,
        (Format::Bin, Format::Txt) => convert::<BinFormat, TxtFormat>(&mut input, &mut stdout)?, 
        _ => println!("input and output formats can not be the same"),
    };
    Ok(())
}