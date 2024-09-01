use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Filepack {
  pub(crate) files: HashMap<Utf8PathBuf, Hash>,
}
