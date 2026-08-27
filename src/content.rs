use super::*;

pub(crate) trait Content: Sized {
  const LABEL: &'static str;

  type Type: ContentType;

  fn cover(&self) -> Option<&Image> {
    None
  }

  fn info(&self, builder: InfoBuilder) -> InfoBuilder;

  fn load(root: &Utf8Path, path: RelativePath) -> Result<Item<Self>>;

  fn path(&self) -> &RelativePath;

  fn resource_type(&self) -> ResourceType {
    self.ty().resource_type()
  }

  #[cfg(test)]
  fn test(path: &str) -> Self;

  fn ty(&self) -> Self::Type;
}
