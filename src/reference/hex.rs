use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Hex {
  pub description: String,
  pub example: String,
  pub example_description: String,
  pub name: String,
  pub schema: String,
}
