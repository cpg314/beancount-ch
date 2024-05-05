use std::path::Path;

#[derive(Default)]
pub struct AccountGuessing(Vec<(String /* description */, String /* account */)>);
impl AccountGuessing {
    pub fn load(filename: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(filename)?;
        Ok(Self(
            data.lines()
                .filter_map(|x| x.split_once(','))
                .map(|(a, b)| (a.trim().to_string(), b.trim().to_string()))
                .collect(),
        ))
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn from_merchant(&self, description: &str) -> String {
        let description = description.to_lowercase();
        for (rule, account) in &self.0 {
            if description.contains(&rule.to_lowercase()) {
                return account.to_string();
            }
        }
        "Liabilities:Unknown".into()
    }
}
