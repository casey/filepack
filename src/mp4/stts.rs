use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Stts {
  pub(crate) entries: Vec<SttsEntry>,
}

impl Stts {
  pub(crate) fn duration(&self) -> u128 {
    self
      .entries
      .iter()
      .map(|entry| u128::from(entry.count) * u128::from(entry.delta))
      .sum()
  }
}

impl Parse for Stts {
  const TYPE: Fourcc = Fourcc(*b"stts");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.version()?;

    let count = reader.u32()?;

    let mut entries = Vec::new();

    for _ in 0..count {
      entries.push(SttsEntry {
        count: reader.u32()?,
        delta: reader.u32()?,
      });
    }

    Ok(Self { entries })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let stts = Stts::parse(Reader::new(&[
      0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6,
    ]))
    .unwrap();

    assert_eq!(
      stts,
      Stts {
        entries: vec![
          SttsEntry { count: 3, delta: 4 },
          SttsEntry { count: 5, delta: 6 },
        ],
      },
    );

    assert_eq!(stts.duration(), 42);

    assert_eq!(
      Stts::parse(Reader::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3]))
        .unwrap_err()
        .to_string(),
      "truncated",
    );
  }
}
