use super::*;

pub(crate) trait MediaItem {
  fn cover(&self) -> Option<&Image>;

  fn info(&self, url: String) -> Info;

  fn path(&self) -> &RelativePath;

  fn resource_type(&self) -> ResourceType;
}
