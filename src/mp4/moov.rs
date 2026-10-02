use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Moov {
  pub(crate) meta: Option<Meta>,
  pub(crate) mvhd: Mvhd,
  pub(crate) traks: Vec<Trak>,
  pub(crate) udta: Option<Udta>,
}

impl Moov {
  pub(crate) fn ilst(&self) -> Option<&Ilst> {
    self
      .udta
      .as_ref()
      .and_then(|udta| udta.meta.as_ref())
      .or(self.meta.as_ref())
      .and_then(|meta| meta.ilst.as_ref())
  }
}

impl Parse for Moov {
  const TYPE: Fourcc = Fourcc(*b"moov");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    let container = reader.container()?;
    Ok(Self {
      meta: container.optional()?,
      mvhd: container.one()?,
      traks: container.many()?,
      udta: container.optional()?,
    })
  }
}
