use std::path::PathBuf;

use itertools::Itertools;
use serde::Deserialize;

use super::beancount::*;
use super::guessing::*;
use super::BeancountParser;

#[derive(clap::Parser)]
/// Revolut statements (CSV format)
pub struct Flags {
    csv: PathBuf,
    #[clap(long, default_value = "Assets:Banks:Revolut")]
    account: String,
}

fn date_deser<'de, D>(deser: D) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let date = String::deserialize(deser)?;
    chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
        .map_err(serde::de::Error::custom)
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "PascalCase")]
struct Row {
    r#type: String,
    product: String,
    #[serde(deserialize_with = "date_deser", rename = "Started Date")]
    started_date: chrono::NaiveDateTime,
    #[serde(rename = "Completed Date")]
    completed_date: Option<String>,
    description: String,
    amount: f32,
    fee: f32,
    currency: String,
    state: String,
    balance: Option<f32>,
}

impl BeancountParser for Flags {
    fn parse(
        &self,
        account_guess: AccountGuessing,
    ) -> anyhow::Result<Vec<crate::beancount::BeancountEntry>> {
        let mut reader = csv::Reader::from_path(&self.csv)?;
        Ok(reader
            .deserialize::<Row>()
            .filter_ok(|row| {
                !row.description.contains("Payment from") && row.completed_date.is_some()
            })
            .map_ok(|row| {
                let account = account_guess.from_merchant(&row.description);
                let lines = vec![
                    BeancountEntryLine::new(&self.account.clone(), Some(row.amount - row.fee)),
                    BeancountEntryLine::new(&account, None),
                ];
                BeancountEntry {
                    description: row.description,
                    date: row.started_date.date(),
                    lines,
                }
            })
            .try_collect()?)
    }
}
