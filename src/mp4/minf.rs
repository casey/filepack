use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Minf {
  pub(crate) stbl: Stbl,
}

impl Parse for Minf {
  const TYPE: Fourcc = Fourcc(*b"minf");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    let container = reader.container()?;
    Ok(Self {
      stbl: container.one()?,
    })
  }
}
