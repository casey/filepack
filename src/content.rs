use super::*;

pub(crate) trait Content: Sized {
  fn info(&self, builder: InfoBuilder) -> InfoBuilder;

  fn load(root: &Utf8Path, path: RelativePath) -> Result<Item<Self>>;

  fn path(&self) -> &RelativePath;

  fn resource_type(&self) -> ResourceType;

  #[cfg(test)]
  fn test(path: &str) -> Self;
}
