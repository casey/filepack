use super::*;

pub(crate) struct Message {
  pub(crate) fingerprint: Hash,
}

impl Message {
  pub(crate) fn digest(self) -> Hash {
    let mut hasher = ContextHasher::new(Context::Message);
    hasher.field(0, self.fingerprint);
    hasher.finalize()
  }
}
