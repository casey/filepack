use super::*;

pub(crate) trait MediaItem {
  fn display_title(&self, index: usize) -> String;

  fn info(&self, url: String) -> Info;

  fn path(&self) -> &RelativePath;

  fn placeholder(&self) -> Option<&Image>;

  fn resource_type(&self) -> ResourceType;

  fn title(&self) -> Option<&Text>;
}
