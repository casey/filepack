use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Data {
  pub(crate) data_type: u32,
  pub(crate) payload: Vec<u8>,
}

impl Data {
  pub(crate) fn pair(&self, tag: Fourcc) -> Result<(u16, u16), Mp4Error> {
    ensure! {
      self.data_type == 0,
      mp4_error::DataType { data_type: self.data_type, tag },
    }

    let mut reader = Reader::new(&self.payload);

    reader
      .skip(2)
      .and_then(|()| Ok((reader.u16()?, reader.u16()?)))
      .ok()
      .context(mp4_error::Pair { tag })
  }

  pub(crate) fn text(&self, tag: Fourcc) -> Result<&str, Mp4Error> {
    ensure! {
      self.data_type == 1,
      mp4_error::DataType { data_type: self.data_type, tag },
    }

    str::from_utf8(&self.payload).context(mp4_error::Utf8 { tag })
  }
}

impl Parse for Data {
  const TYPE: Fourcc = Fourcc(*b"data");

  fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
    let data_type = reader.u32()? & 0x00ff_ffff;
    reader.skip(4)?;

    Ok(Self {
      data_type,
      payload: reader.rest().into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const TAG: Fourcc = Fourcc(*b"trkn");

  fn data(data_type: u32, payload: &[u8]) -> Data {
    Data {
      data_type,
      payload: payload.into(),
    }
  }

  #[test]
  fn pair() {
    assert_eq!(
      data(0, &[0, 0, 0, 3, 0, 4, 0, 0]).pair(TAG).unwrap(),
      (3, 4)
    );

    assert_eq!(
      data(0, &[0, 0, 0, 3]).pair(TAG).unwrap_err().to_string(),
      "invalid `trkn` tag",
    );

    assert_eq!(
      data(1, &[0, 0, 0, 3, 0, 4])
        .pair(TAG)
        .unwrap_err()
        .to_string(),
      "`trkn` tag has unsupported data type 1",
    );
  }

  #[test]
  fn parse() {
    assert_eq!(
      Data::parse(Reader::new(&[1, 0, 0, 13, 0, 0, 0, 0, b'f', b'o', b'o'])).unwrap(),
      data(13, b"foo"),
    );

    assert_eq!(
      Data::parse(Reader::new(&[0; 7])).unwrap_err().to_string(),
      "truncated",
    );
  }

  #[test]
  fn text() {
    assert_eq!(data(1, b"foo").text(TAG).unwrap(), "foo");

    assert_eq!(
      data(1, b"\xff").text(TAG).unwrap_err().to_string(),
      "`trkn` tag is not valid UTF-8",
    );

    assert_eq!(
      data(0, b"foo").text(TAG).unwrap_err().to_string(),
      "`trkn` tag has unsupported data type 0",
    );
  }
}
