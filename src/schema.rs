use super::*;

#[derive(Clone, Copy, Debug, Deserialize, IntoStaticStr, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Schema {
  Boolean,
  Bytes,
  I64,
  String,
  U64,
}

impl Schema {
  pub fn name(self) -> &'static str {
    self.into()
  }

  #[must_use]
  pub fn register(self, schemas: &mut BTreeMap<String, Schema>) -> Self {
    schemas.insert(self.name().into(), self);
    self
  }
}
