use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ContentType {
  pub content_type: String,
  pub extensions: Vec<String>,
  pub sandbox: bool,
}
