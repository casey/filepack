use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Filepack {
  pub(crate) files: HashMap<Utf8PathBuf, Hash>,
}

impl Filepack {
  pub(crate) const FILENAME: &'static str = "filepack.json";
}
