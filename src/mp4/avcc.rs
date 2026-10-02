use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Avcc {
  pub(crate) profile: u8,
  pub(crate) sequence_parameter_sets: Vec<Vec<u8>>,
}

impl Parse for Avcc {
  const TYPE: Fourcc = Fourcc(*b"avcC");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.u8()?;
    let profile = reader.u8()?;
    reader.skip(2)?;
    reader.u8()?;
    let count = reader.u8()? & 0x1f;

    let mut sequence_parameter_sets = Vec::new();

    for _ in 0..count {
      let len = reader.u16()?;
      sequence_parameter_sets.push(reader.bytes(len.into())?.into());
    }

    Ok(Self {
      profile,
      sequence_parameter_sets,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    assert_eq!(
      Avcc::parse(Reader::new(&[1, 100, 0, 0, 0xff, 0xe0, 0])).unwrap(),
      Avcc {
        profile: 100,
        sequence_parameter_sets: Vec::new(),
      },
    );

    assert_eq!(
      Avcc::parse(Reader::new(&[
        1, 100, 0, 0, 0xff, 0xe2, 0, 2, 7, 8, 0, 1, 9, 0
      ]))
      .unwrap(),
      Avcc {
        profile: 100,
        sequence_parameter_sets: vec![vec![7, 8], vec![9]],
      },
    );

    assert_eq!(
      Avcc::parse(Reader::new(&[1, 100, 0, 0, 0xff, 0xe1, 0, 2, 7]))
        .unwrap_err()
        .to_string(),
      "truncated",
    );
  }
}
