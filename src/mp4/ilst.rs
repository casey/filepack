use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Ilst {
  pub(crate) items: Vec<IlstItem>,
}

impl Ilst {
  pub(crate) fn data(&self, tag: Fourcc) -> impl Iterator<Item = &Data> {
    self
      .items
      .iter()
      .filter(move |item| item.ty == tag)
      .flat_map(|item| &item.data)
  }

  pub(crate) fn text(&self, tag: Fourcc) -> Result<Vec<&str>, Mp4Error> {
    self.data(tag).map(|data| data.text(tag)).collect()
  }
}

impl Parse for Ilst {
  const TYPE: Fourcc = Fourcc(*b"ilst");

  fn parse(reader: Reader) -> Result<Self, Mp4Error> {
    Ok(Self {
      items: reader
        .container()?
        .atoms()
        .iter()
        .map(IlstItem::parse)
        .collect::<Result<Vec<IlstItem>, Mp4Error>>()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn text() {
    let bytes = Mp4Builder::new()
      .tag(*b"\xa9nam", "foo")
      .tag(*b"\xa9alb", "bar")
      .tag(*b"\xa9nam", "baz")
      .ilst();

    let ilst = Ilst::parse(Reader::new(&bytes)).unwrap();

    assert_eq!(ilst.text(Fourcc(*b"\xa9nam")).unwrap(), ["foo", "baz"]);
    assert_eq!(ilst.text(Fourcc(*b"\xa9alb")).unwrap(), ["bar"]);
    assert_eq!(ilst.text(Fourcc(*b"\xa9ART")).unwrap(), Vec::<&str>::new());
  }
}
