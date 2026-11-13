use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Response {
  Deco { fields: Vec<deco::Field> },
  File,
  Html,
  Redirect,
  Static,
}
