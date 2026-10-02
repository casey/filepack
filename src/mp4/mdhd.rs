use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Mdhd {
  pub(crate) duration: u64,
  pub(crate) timescale: u32,
}

impl Parse for Mdhd {
  const TYPE: Fourcc = Fourcc(*b"mdhd");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    let (timescale, duration) = match reader.version()? {
      0 => {
        reader.skip(8)?;
        (reader.u32()?, reader.u32()?.into())
      }
      1 => {
        reader.skip(16)?;
        (reader.u32()?, reader.u64()?)
      }
      version => return Err(Mp4Error::Version { version }),
    };
    Ok(Self {
      duration,
      timescale,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    assert_eq!(
      Mdhd::parse(Reader::new(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2
      ]))
      .unwrap(),
      Mdhd {
        duration: 2,
        timescale: 1,
      },
    );

    assert_eq!(
      Mdhd::parse(Reader::new(&[
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 2,
      ]))
      .unwrap(),
      Mdhd {
        duration: 2,
        timescale: 1,
      },
    );

    assert_eq!(
      Mdhd::parse(Reader::new(&[2, 0, 0, 0]))
        .unwrap_err()
        .to_string(),
      "unsupported version 2",
    );
  }
}
