use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MediaFormat {
  pub extension: String,
  pub name: String,
}
