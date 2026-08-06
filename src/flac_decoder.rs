use super::*;

pub(crate) struct FlacDecoder<'a> {
  path: &'a Utf8Path,
  reader: FlacReader<fs::File>,
}

impl<'a> FlacDecoder<'a> {
  fn new(path: &'a Utf8Path) -> Result<Self> {
    let reader = FlacReader::open(path).context(error::FlacDecode { path })?;

    Ok(Self { path, reader })
  }

  fn number_tag(&self, tag: &'static str) -> Result<u64> {
    let value = self.tag(tag)?;
    parse_number(value).context(error::AudioTagInteger {
      path: self.path,
      tag,
    })
  }

  pub(crate) fn read(path: &'a Utf8Path) -> Result<AudioMetadata> {
    let decoder = Self::new(path)?;

    let streaminfo = decoder.reader.streaminfo();

    let samples = streaminfo
      .samples
      .context(error::FlacSampleCountUnknown { path })?;

    Ok(AudioMetadata {
      album: decoder.text_tag("album")?,
      artist: decoder.text_tag("artist")?,
      channels: streaminfo.channels.into(),
      disc: decoder.number_tag("discnumber")?,
      discs: decoder.number_tag("disctotal")?,
      sample_bits: Some(streaminfo.bits_per_sample.into()),
      sample_rate: streaminfo.sample_rate.into(),
      samples,
      title: decoder.text_tag("title")?,
      track: decoder.number_tag("tracknumber")?,
      tracks: decoder.number_tag("tracktotal")?,
    })
  }

  fn tag(&self, tag: &'static str) -> Result<&str> {
    audio_tag(self.reader.get_tag(tag), self.path, tag)
  }

  fn text_tag(&self, tag: &'static str) -> Result<Text> {
    self.tag(tag)?.parse().context(error::AudioTagInvalid {
      path: self.path,
      tag,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_err() {
    fn err(bytes: &[u8]) -> Error {
      let (_tempdir, root) = tempdir();

      let path = root.join("foo.flac");

      std::fs::write(&path, bytes).unwrap();

      FlacDecoder::read(&path).unwrap_err()
    }

    assert_matches!(err(b"foo"), Error::FlacDecode { .. });

    assert_matches!(
      err(&flac(&[], 44100)),
      Error::AudioTagMissing { tag: "album", .. },
    );

    assert_matches!(
      err(&flac(&["ALBUM=qux", "TITLE=bar"], 44100)),
      Error::AudioTagMissing { tag: "artist", .. },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ARTIST=baz", "DISCNUMBER=1", "DISCTOTAL=1"],
        44100,
      )),
      Error::AudioTagMissing { tag: "title", .. },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ALBUM=quux", "ARTIST=baz", "TITLE=bar"],
        44100,
      )),
      Error::AudioTagMultiple { tag: "album", .. },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE="
        ],
        44100,
      )),
      Error::AudioTagEmpty { tag: "title", .. },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=foo\tbar",
        ],
        44100,
      )),
      Error::AudioTagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "title",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
        ],
        44100,
      )),
      Error::AudioTagMissing {
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=foo",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=3/12",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=01",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=+1",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=18446744073709551616",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Integer { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=1",
          "TRACKTOTAL=1",
        ],
        0,
      )),
      Error::FlacSampleCountUnknown { .. },
    );
  }

  #[test]
  fn read_ok() {
    let (_tempdir, root) = tempdir();

    let path = root.join("foo.flac");

    std::fs::write(
      &path,
      flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=2",
          "TITLE=bar",
          "TRACKNUMBER=3",
          "TRACKTOTAL=4",
        ],
        66150,
      ),
    )
    .unwrap();

    assert_eq!(
      FlacDecoder::read(&path).unwrap(),
      AudioMetadata {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        disc: 1,
        discs: 2,
        sample_bits: Some(16),
        sample_rate: 44100,
        samples: 66150,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );
  }
}
