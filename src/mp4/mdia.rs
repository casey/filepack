use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Mdia {
  pub(crate) hdlr: Hdlr,
  pub(crate) mdhd: Mdhd,
  pub(crate) minf: Minf,
}

impl Parse for Mdia {
  const TYPE: Fourcc = Fourcc(*b"mdia");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    let container = reader.container()?;
    Ok(Self {
      hdlr: container.one()?,
      mdhd: container.one()?,
      minf: container.one()?,
    })
  }
}
