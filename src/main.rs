use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;

trait BeancountParser {
    /// Parse statement into a list of Beancount entries
    fn parse(
        &self,
        account_guess: AccountGuessing,
    ) -> anyhow::Result<Vec<beancount::BeancountEntry>>;
}

mod beancount;
mod guessing;
use guessing::*;
mod bcv;
mod cembra;
mod revolut;

/// Parses a monthly statement and output it as Beancount entries
#[derive(Parser)]
struct Flags {
    /// CSV with {text contained in merchant name}, {account} for expense account guessing.
    #[clap(long)]
    accounts_rules_csv: Option<PathBuf>,
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(clap::Subcommand)]
enum Mode {
    Cembra(cembra::Flags),
    Bcv(bcv::Flags),
    Revolut(revolut::Flags),
}
impl Mode {
    fn parser(self) -> Box<dyn BeancountParser> {
        match self {
            Mode::Cembra(x) => Box::new(x),
            Mode::Bcv(x) => Box::new(x),
            Mode::Revolut(x) => Box::new(x),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Flags::parse();

    let account_guess = if let Some(filename) = &args.accounts_rules_csv {
        AccountGuessing::load(filename)?
    } else {
        AccountGuessing::default()
    };
    let parser = args.mode.parser();
    let out = parser.parse(account_guess)?;
    println!("{}", out.into_iter().join("\n"));
    Ok(())
}
