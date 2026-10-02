use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Tkhd {
  pub(crate) matrix: [i32; 9],
}

impl Parse for Tkhd {
  const TYPE: Fourcc = Fourcc(*b"tkhd");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    match reader.version()? {
      0 => reader.skip(20)?,
      1 => reader.skip(32)?,
      version => return Err(Mp4Error::Version { version }),
    }

    reader.skip(16)?;

    let mut matrix = [0; 9];

    for value in &mut matrix {
      *value = reader.i32()?;
    }

    Ok(Self { matrix })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let mut bytes = vec![0; 40];

    for value in 1..=9i32 {
      bytes.extend_from_slice(&value.to_be_bytes());
    }

    assert_eq!(
      Tkhd::parse(Reader::new(&bytes)).unwrap(),
      Tkhd {
        matrix: [1, 2, 3, 4, 5, 6, 7, 8, 9],
      },
    );

    let mut bytes = vec![1];
    bytes.resize(52, 0);

    for value in 1..=9i32 {
      bytes.extend_from_slice(&value.to_be_bytes());
    }

    assert_eq!(
      Tkhd::parse(Reader::new(&bytes)).unwrap(),
      Tkhd {
        matrix: [1, 2, 3, 4, 5, 6, 7, 8, 9],
      },
    );

    assert_eq!(
      Tkhd::parse(Reader::new(&[2, 0, 0, 0]))
        .unwrap_err()
        .to_string(),
      "unsupported version 2",
    );

    assert_eq!(
      Tkhd::parse(Reader::new(&[0; 40])).unwrap_err().to_string(),
      "truncated",
    );
  }
}
