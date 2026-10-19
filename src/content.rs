use super::*;

pub(crate) trait Content: Sized {
  const LABEL: &'static str;

  type Format: Format;

  fn format(&self) -> &Decoded<Self::Format>;

  fn info(&self, builder: InfoBuilder) -> InfoBuilder;

  fn load(root: &Utf8Path, path: RelativePath) -> Result<Item<Self>>;

  fn path(&self) -> &RelativePath;

  fn placeholder(&self) -> Option<&Image> {
    None
  }

  fn resource_type(&self) -> ResourceType {
    self
      .format()
      .known()
      .copied()
      .map_or(ResourceType::Binary, Format::resource_type)
  }

  #[cfg(test)]
  fn test(path: &str) -> Self;
}
