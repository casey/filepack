use super::*;

pub(crate) trait Parse: Sized {
  const TYPE: Fourcc;

  fn parse(reader: Reader) -> Result<Self, Mp4Error>;
}
