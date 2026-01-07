use super::*;

pub(crate) struct FieldHasher {
  array: Option<NonZeroU64>,
  hasher: Hasher,
  next: u8,
}

impl FieldHasher {
  pub(crate) fn array(&mut self, tag: u8, len: u64) {
    self.tag(tag);
    self.array = NonZeroU64::new(len);
    self.hasher.update(&len.to_le_bytes());
  }

  pub(crate) fn element(&mut self, hash: Hash) {
    self.array = NonZeroU64::new(self.array.unwrap().get() - 1);
    self.hasher.update(hash.as_bytes());
  }

  pub(crate) fn field(&mut self, tag: u8, contents: &[u8]) {
    self.tag(tag);
    self.hasher.update(&contents.len().into_u64().to_le_bytes());
    self.hasher.update(contents);
  }

  pub(crate) fn finalize(self) -> Hash {
    assert_eq!(self.array, None);
    self.hasher.finalize().into()
  }

  pub(crate) fn new(context: Context) -> Self {
    Self {
      array: None,
      hasher: Hasher::new_derive_key(&format!("filepack:{context}")),
      next: 0,
    }
  }

  pub(crate) fn tag(&mut self, tag: u8) {
    assert_eq!(self.next, tag);
    assert_eq!(self.array, None);
    self.next = self.next.checked_add(1).unwrap();
    self.hasher.update(&[tag]);
  }
}
