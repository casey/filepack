use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Hex {
  pub example: String,
  pub name: String,
  pub schema: String,
}
