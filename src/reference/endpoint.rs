use super::*;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Endpoint {
  pub authenticated: bool,
  pub description: String,
  pub parameters: Vec<Parameter>,
  pub query: Vec<QueryParameter>,
  pub range: bool,
  pub request: Option<Request>,
  pub response: Option<Response>,
}
