use super::*;

pub(crate) trait Content {
  fn info(&self, builder: InfoBuilder) -> InfoBuilder;

  fn path(&self) -> &RelativePath;

  fn resource_type(&self) -> ResourceType;
}
