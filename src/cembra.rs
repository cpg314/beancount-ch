use std::path::PathBuf;

use super::beancount::*;
use super::guessing::*;
use super::BeancountParser;

#[derive(clap::Parser)]
/// Monthly credit card statements from Cembra (PDF format)
pub struct Flags {
    pdf: PathBuf,
    #[clap(long, default_value = "Liabilities:CreditCard")]
    cc_account: String,
}

fn get_element(node: &xmltree::XMLNode) -> &xmltree::Element {
    match node {
        xmltree::XMLNode::Element(e) => e,
        _ => panic!(),
    }
}
impl BeancountParser for Flags {
    fn parse(
        &self,
        account_guess: AccountGuessing,
    ) -> anyhow::Result<Vec<crate::beancount::BeancountEntry>> {
        // Convert the PDF into XML
        let doc = duct::cmd!("pdftohtml", &self.pdf, "-xml", "-f", "2", "-stdout").read()?;
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
                                            BeancountEntryLine::new(&self.cc_account, None),
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
}
