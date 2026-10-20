use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Parameter {
  Fingerprint,
  Hash,
  Integer,
  Ordinal,
  PackageIdentifier,
  RelativePath,
  Revision,
  String,
}
