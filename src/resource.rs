use super::*;

pub(crate) struct Resource {
  pub(crate) content_length: u64,
  pub(crate) content_type: Mime,
  pub(crate) file: fs::File,
  pub(crate) hash: Hash,
}

impl Resource {
  pub(crate) fn with_content_type(self, content_type: Mime) -> Self {
    Self {
      content_type,
      ..self
    }
  }
}
