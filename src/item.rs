use super::*;

pub(crate) trait Item {
  fn path(&self) -> RelativePath;

  fn properties(&self) -> Vec<(String, Value)>;

  fn resource_type(&self) -> ResourceType;
}
