use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
  pub(crate) files: HashMap<Utf8PathBuf, Hash>,
}

impl Manifest {
  pub(crate) const FILENAME: &'static str = "filepack.json";
}
