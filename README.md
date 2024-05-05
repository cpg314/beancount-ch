# beancount-ch

Convert monthly statements from some banks serving the Swiss market into [beancount](https://github.com/beancount/beancount) plain-text entries.

- [Cembra](https://www.cembra.ch/) credit card statements (PDFs)
- [Revolut](https://www.revolut.com/) statements (CSVs)
- [BCV](https://www.bcv.ch/) monthly statements (XLSX spreadsheets)

See https://c.pgdm.ch/code/beancount-ch

Note that beancount itself has importation tools: see [this documentation page](https://beancount.github.io/docs/importing_external_data.html) and [beangulp](https://github.com/beancount/beangulp).

## Usage

```console
beancount-ch
Parses a monthly statement and output it as Beancount entries

USAGE:
    beancount-ch [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --accounts-rules-csv <ACCOUNTS_RULES_CSV>
            CSV with {text contained in merchant name}, {account} for expense account guessing

    -h, --help
            Print help information

SUBCOMMANDS:
    bcv        Monthly statements from BCV (XLSX format)
    cembra     Monthly credit card statements from Cembra (PDF format)
    help       Print this message or the help of the given subcommand(s)
    revolut    Revolut statements (CSV format)
```

A command of the form

```console
$ cembra-beancount --accounts-rules-csv rules.csv cembra 2022-05.pdf
```

then produces beancount entries of the form

```beancount
2022-05-01 * "Bakery Inc."
 Expenses:Food 15.80 CHF
 Liabilities:CreditCard
...
```

### Account guessing

In many cases, the expense account (here `Expenses:Food`) can be guessed from the merchant name. The `--accounts-rules-csv` file contains simple rules of the form

```text
 {text contained in merchant name}, {account}
 ...
```

applied sequentially until a match is found.
