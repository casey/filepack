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
    let mut registry = json::Registry::new();

    let schema = T::json_schema(&mut registry);

    let json::Schema::StringEnum {
      description,
      values,
    } = &registry[&schema]
    else {
      unreachable!()
    };

    Self {
      default: T::default().to_string(),
      description: description.clone(),
      name: name.into(),
      values: values.clone(),
    }
  }
}
