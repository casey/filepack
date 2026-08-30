use super::*;

pub(crate) trait ContentType: Copy + Display + PartialEq {
  const EXTENSIONS: &'static [&'static str];

  fn from_extension(extension: &str) -> Option<Self>;

  fn from_path(path: &RelativePath) -> Result<Self, PathError> {
    path
      .extension()
      .and_then(Self::from_extension)
      .ok_or(PathError::Extension {
        extensions: Self::EXTENSIONS,
      })
  }

  fn resource_type(self) -> ResourceType;
}
