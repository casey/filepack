use super::*;

#[derive(Clone, Copy, Debug, Display, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum UnstableFeature {
  WebPackages,
}
