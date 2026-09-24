use super::*;

#[derive(Debug, Decode, Encode, PartialEq)]
#[deco(transparent)]
pub struct Revision(pub(crate) Hash);

impl Display for Revision {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.format(f)
  }
}

impl Hex for Revision {
  const TAG: Tag = Tag::Revision;
}

impl FromStr for Revision {
  type Err = HexError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl From<Hash> for Revision {
  fn from(hash: Hash) -> Self {
    Self(hash)
  }
}
