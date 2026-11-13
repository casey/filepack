use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PathParameter {
  pub description: String,
  pub name: String,
}
