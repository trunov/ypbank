# ypbank
Rust crates for parsing, converting, and comparing financial transaction data across multiple formats (CSV, text, binary).

## Workspace Structure

| Crate       | Type    | Description                                      |
|-------------|---------|--------------------------------------------------|
| `ypbank`    | library | Core parsing and serialisation logic             |
| `converter` | binary  | Converts transaction files between formats       |
| `comparer`  | binary  | Compares two transaction files for differences   |

## Supported Formats

| Value    | Description                          |
|----------|--------------------------------------|
| `csv`    | Comma-separated values               |
| `txt`    | Human-readable plain text            |
| `binary` | Compact binary format                |

---

## converter

Reads a transaction file in one format and writes the result to stdout in another.

### Usage

```
converter --input <FILE> --input-format <FORMAT> --output-format <FORMAT>
```

### Arguments

| Argument          | Values                  | Description          |
|-------------------|-------------------------|----------------------|
| `--input`         | path                    | Input file path      |
| `--input-format`  | `csv`, `txt`, `binary`  | Format of input file |
| `--output-format` | `csv`, `txt`, `binary`  | Format of output     |

### Examples

Convert a plain text file to CSV and save to a file:
```bash
cargo run -p converter -- --input tx.txt --input-format txt --output-format csv > output.csv
```

Convert a binary file to plain text:
```bash
cargo run -p converter -- --input tx.bin --input-format binary --output-format txt
```

Convert CSV to binary:
```bash
cargo run -p converter -- --input tx.csv --input-format csv --output-format binary > output.bin
```

---

## comparer

Compares two transaction files (potentially in different formats) and reports any differences.

Differences are reported in three categories:
- Transactions missing in file 1
- Transactions missing in file 2
- Transactions present in both files but with differing fields

### Usage

```
comparer --file1 <FILE> --format1 <FORMAT> --file2 <FILE> --format2 <FORMAT>
```

### Arguments

| Argument    | Values                  | Description           |
|-------------|-------------------------|-----------------------|
| `--file1`   | path                    | First file path       |
| `--format1` | `csv`, `txt`, `binary`  | Format of first file  |
| `--file2`   | path                    | Second file path      |
| `--format2` | `csv`, `txt`, `binary`  | Format of second file |

### Examples

Compare a binary file against a CSV file:
```bash
cargo run -p comparer -- --file1 tx.bin --format1 binary --file2 tx.csv --format2 csv
```

Compare two CSV files:
```bash
cargo run -p comparer -- --file1 old.csv --format1 csv --file2 new.csv --format2 csv
```

Compare a text file against a binary file:
```bash
cargo run -p comparer -- --file1 tx.txt --format1 txt --file2 tx.bin --format2 binary
```

### Example Output

When files are identical:
```
The transaction records in 'tx.bin' and 'tx.csv' are identical.
```

When differences are found:
```
Transaction 42 is missing in 'old.csv'
Transaction 7 is missing in 'new.csv'
Transaction 3 differs between 'old.csv' and 'new.csv':
  old.csv: Transaction { tx_id: 3, amount: 1000, ... }
  new.csv: Transaction { tx_id: 3, amount: 9999, ... }
```

---

## Building

Build all crates from the workspace root:
```bash
cargo build
```

Run all tests:
```bash
cargo test
```