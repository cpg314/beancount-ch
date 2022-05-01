# cembra-beancount

Convert Cembra's monthly credit card statements into [beancount](https://github.com/beancount/beancount) entries using Rust and pdftohtml.

```
Parse a Cembra PDF monthly statement and output it as Beancount

USAGE:
    cembra-beancount [OPTIONS] <PDF>

OPTIONS:
        --accounts-rules-csv <ACCOUNTS_RULES_CSV>
            CSV with {text contained in merchant name}, {account} for expense account
            guessing

        --cc-account <CC_ACCOUNT>
            [default: Liabilities:CreditCard]
```

See https://c.pgdm.ch/notes/cembra-beancount/
