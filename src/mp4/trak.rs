use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Trak {
  pub(crate) mdia: Mdia,
  pub(crate) tkhd: Tkhd,
}

impl Parse for Trak {
  const TYPE: Fourcc = Fourcc(*b"trak");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    let container = reader.container()?;

    Ok(Self {
      mdia: container.one()?,
      tkhd: container.one()?,
    })
  }
}
