use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Request {
  Bytes,
  Deco {
    fields: Vec<deco::Field>,
    limit: u64,
  },
}
