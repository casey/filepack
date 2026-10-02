use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Hdlr {
  pub(crate) handler_type: Fourcc,
}

impl Hdlr {
  pub(crate) fn name(&self) -> &'static str {
    match &self.handler_type.0 {
      b"auxv" => "auxiliary video",
      b"meta" => "metadata",
      b"pict" => "picture",
      b"soun" => "audio",
      b"vide" => "video",
      _ => "unknown",
    }
  }
}

impl Parse for Hdlr {
  const TYPE: Fourcc = Fourcc(*b"hdlr");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.version()?;
    reader.skip(4)?;
    Ok(Self {
      handler_type: reader.fourcc()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn name() {
    for (handler_type, expected) in [
      (*b"auxv", "auxiliary video"),
      (*b"meta", "metadata"),
      (*b"pict", "picture"),
      (*b"soun", "audio"),
      (*b"vide", "video"),
      (*b"xxxx", "unknown"),
    ] {
      assert_eq!(
        Hdlr {
          handler_type: Fourcc(handler_type),
        }
        .name(),
        expected,
      );
    }
  }

  #[test]
  fn parse() {
    assert_eq!(
      Hdlr::parse(Reader::new(&[
        0, 0, 0, 0, 0, 0, 0, 0, b's', b'o', b'u', b'n'
      ]))
      .unwrap(),
      Hdlr {
        handler_type: Fourcc(*b"soun"),
      },
    );
  }
}
