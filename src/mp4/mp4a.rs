use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Mp4a {
  pub(crate) channels: u16,
  pub(crate) esds: Esds,
  pub(crate) sample_rate: u32,
}

impl Parse for Mp4a {
  const TYPE: Fourcc = Fourcc(*b"mp4a");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    reader.skip(8)?;
    let version = reader.u16()?;
    reader.skip(6)?;
    let channels = reader.u16()?;
    reader.skip(6)?;
    let sample_rate = reader.u32()? >> 16;

    match version {
      0 => {}
      1 => reader.skip(16)?,
      version => {
        return Err(Mp4Error::Version {
          version: u8::try_from(version).unwrap_or(u8::MAX),
        });
      }
    }

    Ok(Self {
      channels,
      esds: reader.container()?.one()?,
      sample_rate,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn entry(version: u16) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0; 8]);
    bytes.extend_from_slice(&version.to_be_bytes());
    bytes.extend_from_slice(&[0; 6]);
    bytes.extend_from_slice(&2u16.to_be_bytes());
    bytes.extend_from_slice(&[0; 6]);
    bytes.extend_from_slice(&(44100u32 << 16).to_be_bytes());
    if version == 1 {
      bytes.extend_from_slice(&[0; 16]);
    }
    bytes.extend_from_slice(&Mp4Builder::atom(
      *b"esds",
      &[
        0, 0, 0, 0, 3, 18, 0, 1, 0, 4, 13, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      ],
    ));
    bytes
  }

  #[test]
  fn parse() {
    for version in [0, 1] {
      assert_eq!(
        Mp4a::parse(Reader::new(&entry(version))).unwrap(),
        Mp4a {
          channels: 2,
          esds: Esds { object_type: 0x40 },
          sample_rate: 44100,
        },
      );
    }

    assert_eq!(
      Mp4a::parse(Reader::new(&entry(2))).unwrap_err().to_string(),
      "unsupported version 2",
    );
  }
}
