use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Udta {
  pub(crate) meta: Option<Meta>,
}

impl Parse for Udta {
  const TYPE: Fourcc = Fourcc(*b"udta");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    let container = reader.container()?;

    Ok(Self {
      meta: container.optional()?,
    })
  }
}
