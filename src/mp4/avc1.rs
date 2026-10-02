use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Avc1 {
  pub(crate) avcc: Avcc,
  pub(crate) height: u16,
  pub(crate) width: u16,
}

impl Parse for Avc1 {
  const TYPE: Fourcc = Fourcc(*b"avc1");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.skip(24)?;
    let width = reader.u16()?;
    let height = reader.u16()?;
    reader.skip(50)?;
    Ok(Self {
      avcc: reader.container()?.one()?,
      height,
      width,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let entry = Mp4Builder::video_entry(*b"avc1", *b"avcC", &[1, 100, 0, 0, 0xff, 0xe0, 0], 2, 1);

    assert_eq!(
      Avc1::parse(Reader::new(&entry[8..])).unwrap(),
      Avc1 {
        avcc: Avcc {
          profile: 100,
          sequence_parameter_sets: Vec::new(),
        },
        height: 1,
        width: 2,
      },
    );
  }
}
