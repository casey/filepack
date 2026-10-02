use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct BoxHeader {
  pub(crate) len: u64,
  pub(crate) size: Option<u64>,
  pub(crate) ty: Fourcc,
}

impl BoxHeader {
  pub(crate) fn parse(reader: &mut Reader) -> Result<Self, Mp4Error> {
    let size = reader.u32()?;
    let ty = reader.fourcc()?;

    Ok(match size {
      0 => Self {
        len: 8,
        size: None,
        ty,
      },
      1 => {
        let size = reader.u64()?;
        ensure!(size >= 16, mp4_error::Size { size });
        Self {
          len: 16,
          size: Some(size),
          ty,
        }
      }
      size => {
        ensure!(size >= 8, mp4_error::Size { size });
        Self {
          len: 8,
          size: Some(size.into()),
          ty,
        }
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    #[track_caller]
    fn case(bytes: &[u8], expected: Result<BoxHeader, &str>) {
      assert_eq!(
        BoxHeader::parse(&mut Reader::new(bytes)).map_err(|err| err.to_string()),
        expected.map_err(ToString::to_string),
      );
    }

    case(
      &[0, 0, 0, 9, b'f', b'o', b'o', b'b'],
      Ok(BoxHeader {
        len: 8,
        size: Some(9),
        ty: Fourcc(*b"foob"),
      }),
    );

    case(
      &[0, 0, 0, 0, b'f', b'o', b'o', b'b'],
      Ok(BoxHeader {
        len: 8,
        size: None,
        ty: Fourcc(*b"foob"),
      }),
    );

    case(
      &[0, 0, 0, 1, b'f', b'o', b'o', b'b', 0, 0, 0, 0, 0, 0, 0, 17],
      Ok(BoxHeader {
        len: 16,
        size: Some(17),
        ty: Fourcc(*b"foob"),
      }),
    );

    case(
      &[0, 0, 0, 7, b'f', b'o', b'o', b'b'],
      Err("invalid box size 7"),
    );

    case(
      &[0, 0, 0, 1, b'f', b'o', b'o', b'b', 0, 0, 0, 0, 0, 0, 0, 15],
      Err("invalid box size 15"),
    );

    case(&[0, 0, 0, 1, b'f', b'o', b'o', b'b'], Err("truncated"));

    case(&[0, 0, 0], Err("truncated"));
  }
}
