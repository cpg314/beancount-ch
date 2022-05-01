use std::path::{Path, PathBuf};

use clap::Parser;
use itertools::Itertools;

/// Parse a Cembra PDF monthly statement and output it as Beancount
#[derive(Parser)]
struct Flags {
    /// CSV with {text contained in merchant name}, {account} for expense account guessing.
    #[clap(long)]
    accounts_rules_csv: Option<PathBuf>,
    pdf: PathBuf,
    #[clap(long, default_value = "Liabilities:CreditCard")]
    cc_account: String,
}

use beancount::*;
/// Beancount entries, of the form
/// {YYY-MM-DD} * "{description}"
///  {account} {amount} {currency}
///  ...
mod beancount {
    use std::fmt::{self, Display};
    pub struct BeancountEntryLine {
        account: String,
        amount: Option<f32>,
    }
    impl BeancountEntryLine {
        pub fn new(account: &str, amount: Option<f32>) -> Self {
            Self {
                account: account.into(),
                amount,
            }
        }
    }
    impl Display for BeancountEntryLine {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.account)?;
            if let Some(amount) = self.amount {
                write!(f, " {:.2} CHF", amount)?;
            }
            Ok(())
        }
    }
    pub struct BeancountEntry {
        pub description: String,
        pub date: chrono::NaiveDate,
        pub lines: Vec<BeancountEntryLine>,
    }
    impl Display for BeancountEntry {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            writeln!(
                f,
                "{} * \"{}\"",
                self.date.format("%Y-%m-%d"),
                self.description
            )?;
            for l in &self.lines {
                writeln!(f, " {}", l)?;
            }
            Ok(())
        }
    }
}
fn get_element(node: &xmltree::XMLNode) -> &xmltree::Element {
    match node {
        xmltree::XMLNode::Element(e) => e,
        _ => panic!(),
    }
}
#[derive(Default)]
struct AccountGuessing(Vec<(String /* description */, String /* account */)>);
impl AccountGuessing {
    fn load(filename: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(filename)?;
        Ok(Self(
            data.lines()
                .filter_map(|x| x.split_once(','))
                .map(|(a, b)| (a.trim().to_string(), b.trim().to_string()))
                .collect(),
        ))
    }
    #[allow(clippy::wrong_self_convention)]
    fn from_merchant(&self, description: &str) -> String {
        let description = description.to_lowercase();
        for (rule, account) in &self.0 {
            if description.contains(&rule.to_lowercase()) {
                return account.to_string();
            }
        }
        "??".into()
    }
}
/// Parse statement into a list of Beancount entries
fn parse(args: &Flags) -> anyhow::Result<Vec<BeancountEntry>> {
    // Read rules
    let account_guess = if let Some(filename) = &args.accounts_rules_csv {
        AccountGuessing::load(filename)?
    } else {
        AccountGuessing::default()
    };
    // Convert the PDF into XML
    let doc = duct::cmd!("pdftohtml", &args.pdf, "-xml", "-f", "2", "-stdout").read()?;
    // Parse the XML
    let doc = xmltree::Element::parse(doc.as_bytes())?;
    let pages = doc.children;
    pages
        .into_iter()
        .flat_map(|page| {
            let page = get_element(&page);
            // The entries appear as groups of 4 nodes:
            // <text>{Transaction date}</text>
            // <text>{Accounting date}</text>
            // <text>{Merchant}</text>
            // <text>{Amount}</text>
            page.children
                // Overlapping windows
                .windows(4)
                .filter_map(|children| -> Option<anyhow::Result<BeancountEntry>> {
                    let children: Vec<_> = children
                        .iter()
                        .map(get_element)
                        .flat_map(|c| &c.children)
                        .filter_map(|c| match c {
                            xmltree::XMLNode::Text(t) => Some(t),
                            _ => None,
                        })
                        .collect();
                    if children.len() != 4 {
                        return None;
                    }
                    // Date
                    const DATE_FMT: &str = "%d.%m.%Y";
                    if let (Ok(date1), Ok(_)) = (
                        chrono::NaiveDate::parse_from_str(children[0], DATE_FMT),
                        chrono::NaiveDate::parse_from_str(children[1], DATE_FMT),
                    ) {
                        // Merchant and amount
                        return Some(
                            children[3]
                                .parse::<f32>()
                                .map_err(|e| anyhow::anyhow!(e))
                                .map(|amount| BeancountEntry {
                                    description: children[2].to_string(),
                                    date: date1,
                                    lines: vec![
                                        BeancountEntryLine::new(
                                            &account_guess.from_merchant(children[2]),
                                            Some(amount),
                                        ),
                                        BeancountEntryLine::new(&args.cc_account, None),
                                    ],
                                }),
                        );
                    }
                    None
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
fn main() -> anyhow::Result<()> {
    let args = Flags::parse();
    println!("{}", parse(&args)?.into_iter().join("\n"));
    Ok(())
}
