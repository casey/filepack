use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Ftyp;

impl Parse for Ftyp {
  const TYPE: Fourcc = Fourcc(*b"ftyp");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.fourcc()?;
    reader.u32()?;
    Ok(Self)
  }
}
