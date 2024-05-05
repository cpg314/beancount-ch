//! Beancount entries, of the form
//! {YYY-MM-DD} * "{description}"
//!  {account} {amount} {currency}
//!  ...

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
