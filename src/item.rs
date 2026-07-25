use super::*;

pub(crate) trait Item {
  fn info(&self) -> Info;

  fn path(&self) -> RelativePath;

  fn resource_type(&self) -> ResourceType;
}
