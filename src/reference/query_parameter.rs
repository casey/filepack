use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct QueryParameter {
  pub default: String,
  pub description: String,
  pub name: String,
  pub values: Vec<json::Value>,
}

impl QueryParameter {
  pub(crate) fn new<T: Default + Display + Json>(name: &str) -> Self {
    let schema = T::schema();

    Self {
      default: T::default().to_string(),
      description: schema.description().unwrap().into(),
      name: name.into(),
      values: schema.values().to_vec(),
    }
  }
}
