use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Stsz {
  pub(crate) sample_count: u32,
  pub(crate) sample_size: u32,
  pub(crate) sizes: Vec<u32>,
}

impl Stsz {
  pub(crate) fn size(&self) -> u64 {
    if self.sample_size == 0 {
      self.sizes.iter().copied().map(u64::from).sum()
    } else {
      u64::from(self.sample_size) * u64::from(self.sample_count)
    }
  }
}

impl Parse for Stsz {
  const TYPE: Fourcc = Fourcc(*b"stsz");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.version()?;

    let sample_size = reader.u32()?;
    let sample_count = reader.u32()?;

    let mut sizes = Vec::new();

    if sample_size == 0 {
      for _ in 0..sample_count {
        sizes.push(reader.u32()?);
      }
    }

    Ok(Self {
      sample_count,
      sample_size,
      sizes,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let stsz = Stsz::parse(Reader::new(&[0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 3])).unwrap();

    assert_eq!(
      stsz,
      Stsz {
        sample_count: 3,
        sample_size: 5,
        sizes: Vec::new(),
      },
    );

    assert_eq!(stsz.size(), 15);

    let stsz = Stsz::parse(Reader::new(&[
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 5,
    ]))
    .unwrap();

    assert_eq!(
      stsz,
      Stsz {
        sample_count: 2,
        sample_size: 0,
        sizes: vec![3, 5],
      },
    );

    assert_eq!(stsz.size(), 8);

    assert_eq!(
      Stsz::parse(Reader::new(&[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3
      ]))
      .unwrap_err()
      .to_string(),
      "truncated",
    );
  }
}
