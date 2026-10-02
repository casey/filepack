use super::*;

#[derive(Debug, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum UnstableFeature {
  WebPackages,
}
