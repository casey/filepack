use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Stbl {
  pub(crate) stsd: Stsd,
  pub(crate) stsz: Stsz,
  pub(crate) stts: Stts,
}

impl Parse for Stbl {
  const TYPE: Fourcc = Fourcc(*b"stbl");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    let container = reader.container()?;
    Ok(Self {
      stsd: container.one()?,
      stsz: container.one()?,
      stts: container.one()?,
    })
  }
}
