use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
  pub api: bool,
  pub methods: BTreeMap<String, Endpoint>,
  pub parameters: Vec<PathParameter>,
  pub wildcard: bool,
}
