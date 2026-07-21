use super::*;

pub(crate) trait Item {
  fn path(&self) -> RelativePath;

  fn resource_type(&self) -> ResourceType;
}
