use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Stsd {
  pub(crate) entries: Vec<SampleEntry>,
}

impl Parse for Stsd {
  const TYPE: Fourcc = Fourcc(*b"stsd");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.version()?;
    reader.skip(4)?;

    Ok(Self {
      entries: reader
        .container()?
        .atoms()
        .iter()
        .map(SampleEntry::parse)
        .collect::<Result<Vec<SampleEntry>, Mp4Error>>()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let bytes = [
      vec![0; 8],
      Mp4Builder::atom(*b"fLaC", &[]),
      Mp4Builder::atom(*b"Opus", &[]),
    ]
    .concat();

    assert_eq!(
      Stsd::parse(Reader::new(&bytes)).unwrap(),
      Stsd {
        entries: vec![
          SampleEntry::Unknown(Fourcc(*b"fLaC")),
          SampleEntry::Unknown(Fourcc(*b"Opus")),
        ],
      },
    );
  }
}
