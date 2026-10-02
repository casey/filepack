use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Atom<'a> {
  pub(crate) body: &'a [u8],
  pub(crate) ty: Fourcc,
}

impl Atom<'_> {
  pub(crate) fn parse<T: Parse>(&self) -> Result<T, Mp4Error> {
    T::parse(Reader::new(self.body)).context(mp4_error::Invalid { ty: self.ty })
  }
}
