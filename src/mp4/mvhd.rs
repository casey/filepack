use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Mvhd {
  pub(crate) duration: u64,
  pub(crate) timescale: u32,
}

impl Parse for Mvhd {
  const TYPE: Fourcc = Fourcc(*b"mvhd");

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
    #[track_caller]
    fn case(bytes: &[u8], expected: Result<Mvhd, &str>) {
      assert_eq!(
        Mvhd::parse(Reader::new(bytes)).map_err(|err| err.to_string()),
        expected.map_err(ToString::to_string),
      );
    }

    case(
      &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2],
      Ok(Mvhd {
        duration: 2,
        timescale: 1,
      }),
    );

    case(
      &[
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 2,
      ],
      Ok(Mvhd {
        duration: 2,
        timescale: 1,
      }),
    );

    case(&[2, 0, 0, 0], Err("unsupported version 2"));

    case(&[0, 0, 0, 0, 0, 0, 0, 0], Err("truncated"));
  }
}
