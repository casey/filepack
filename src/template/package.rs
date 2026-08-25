use super::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Package {
  pub(crate) colophon: Option<RelativePath>,
  pub(crate) creator: Option<Text>,
  pub(crate) description: Option<Text>,
  pub(crate) homepage: Option<CheckedUrl>,
  pub(crate) time: Option<Time>,
  pub(crate) title: Option<Text>,
}

impl From<Package> for crate::Package {
  fn from(package: Package) -> Self {
    let Package {
      colophon,
      creator,
      description,
      homepage,
      time,
      title,
    } = package;

    Self {
      colophon,
      creator,
      description,
      homepage,
      time,
      title,
    }
  }
}
