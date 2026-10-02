use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Esds {
  pub(crate) object_type: u8,
}

impl Esds {
  fn descriptor<'a>(reader: &mut Reader<'a>, tag: u8) -> Result<Reader<'a>, Mp4Error> {
    ensure!(reader.u8()? == tag, mp4_error::Descriptor { tag });

    let mut len = 0;
    let mut bytes = 0;

    loop {
      let byte = reader.u8()?;
      bytes += 1;
      len = len << 7 | usize::from(byte & 0x7f);
      if byte & 0x80 == 0 {
        break;
      }
      ensure!(bytes < 4, mp4_error::Descriptor { tag });
    }

    Ok(Reader::new(reader.bytes(len)?))
  }
}

impl Parse for Esds {
  const TYPE: Fourcc = Fourcc(*b"esds");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.version()?;

    let mut es = Self::descriptor(&mut reader, 0x03)?;

    es.u16()?;
    let flags = es.u8()?;

    if flags & 0x80 != 0 {
      es.u16()?;
    }

    if flags & 0x40 != 0 {
      let len = es.u8()?;
      es.skip(len.into())?;
    }

    if flags & 0x20 != 0 {
      es.u16()?;
    }

    let mut decoder_config = Self::descriptor(&mut es, 0x04)?;

    Ok(Self {
      object_type: decoder_config.u8()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    #[track_caller]
    fn case(bytes: &[u8], expected: Result<u8, &str>) {
      assert_eq!(
        Esds::parse(Reader::new(bytes))
          .map(|esds| esds.object_type)
          .map_err(|err| err.to_string()),
        expected.map_err(ToString::to_string),
      );
    }

    case(&[0, 0, 0, 0, 3, 6, 0, 1, 0, 4, 1, 0x40], Ok(0x40));

    case(
      &[
        0, 0, 0, 0, 3, 0x80, 0x80, 0x80, 9, 0, 1, 0, 4, 0x80, 0x80, 0x80, 1, 0x6b,
      ],
      Ok(0x6b),
    );

    case(
      &[
        0, 0, 0, 0, 3, 12, 0, 1, 0xe0, 0, 2, 1, b'x', 0, 3, 4, 1, 0x40,
      ],
      Ok(0x40),
    );

    case(&[0, 0, 0, 0, 4, 1, 0], Err("invalid descriptor 0x03"));

    case(
      &[0, 0, 0, 0, 3, 5, 0, 1, 0, 5, 1, 0],
      Err("invalid descriptor 0x04"),
    );

    case(
      &[0, 0, 0, 0, 3, 0x80, 0x80, 0x80, 0x80, 1],
      Err("invalid descriptor 0x03"),
    );

    case(&[0, 0, 0, 0, 3, 5, 0, 1], Err("truncated"));
  }
}
