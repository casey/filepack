use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MediaFormat {
  pub description: String,
  pub extension: String,
  pub name: String,
}
