use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LintGroup {
  pub description: String,
  pub lints: BTreeSet<Lint>,
}
