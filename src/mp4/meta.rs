use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Meta {
  pub(crate) ilst: Option<Ilst>,
}

impl Parse for Meta {
  const TYPE: Fourcc = Fourcc(*b"meta");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.version()?;
    let container = reader.container()?;
    Ok(Self {
      ilst: container.optional()?,
    })
  }
}
