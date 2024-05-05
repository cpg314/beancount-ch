use std::path::PathBuf;

use itertools::Itertools;
use serde::Deserialize;

use super::beancount::*;
use super::guessing::*;
use super::BeancountParser;

#[derive(clap::Parser)]
/// Monthly statements from BCV (XLSX format)
pub struct Flags {
    xlsx: PathBuf,
    #[clap(long, default_value = "Assets:Banks:BCV")]
    account: String,
}

fn date_deser<'de, D>(deser: D) -> Result<chrono::NaiveDate, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let date = String::deserialize(deser)?;
    chrono::NaiveDate::parse_from_str(&date, "%d.%m.%Y").map_err(serde::de::Error::custom)
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Row {
    #[serde(deserialize_with = "date_deser")]
    execution_date: chrono::NaiveDate,
    description: String,
    debit: Option<f32>,
    credit: Option<f32>,
    #[serde(deserialize_with = "date_deser")]
    value_date: chrono::NaiveDate,
    solde: Option<f64>,
}
impl BeancountParser for Flags {
    fn parse(
        &self,
        account_guess: AccountGuessing,
    ) -> anyhow::Result<Vec<crate::beancount::BeancountEntry>> {
        let doc = duct::cmd!(
            "ssconvert",
            &self.xlsx,
            "-T",
            "Gnumeric_stf:stf_csv",
            "fd://1"
        )
        .read()?;
        let mut lines = doc.lines();
        for l in lines.by_ref() {
            if l.starts_with("\"Date d'exécution") {
                break;
            }
        }
        let lines = std::io::Cursor::new(lines.join("\n"));
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(lines);
        Ok(reader
            .deserialize::<Row>()
            .map_ok(|row| {
                let account = account_guess.from_merchant(&row.description);
                let mut lines = vec![];
                if let Some(debit) = row.debit {
                    lines.push(BeancountEntryLine::new(&self.account.clone(), Some(-debit)));
                    lines.push(BeancountEntryLine::new(&account, None));
                } else if let Some(credit) = row.credit {
                    lines.push(BeancountEntryLine::new(&self.account.clone(), Some(credit)));
                    lines.push(BeancountEntryLine::new(&account, None));
                }
                BeancountEntry {
                    description: row.description,
                    date: row.execution_date,
                    lines,
                }
            })
            .try_collect()?)
    }
}
