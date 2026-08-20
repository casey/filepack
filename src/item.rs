use super::*;

pub(crate) trait Item {
  fn info(&self, url: String) -> Info;

  fn path(&self) -> &RelativePath;

  fn resource_type(&self) -> ResourceType;
}
