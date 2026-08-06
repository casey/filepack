use super::*;

pub(crate) struct FlacDecoder<'a> {
  reader: FlacReader<&'a [u8]>,
}

impl<'a> FlacDecoder<'a> {
  fn decode(data: &'a [u8]) -> Result<AudioMetadata, AudioError> {
    let decoder = Self {
      reader: FlacReader::new(data).context(audio_error::FlacDecode)?,
    };

    let streaminfo = decoder.reader.streaminfo();

    let samples = streaminfo
      .samples
      .context(audio_error::FlacSampleCountUnknown)?;

    Ok(AudioMetadata {
      album: decoder.text_tag("album")?,
      artist: decoder.text_tag("artist")?,
      channels: streaminfo.channels.into(),
      disc: decoder.number_tag("discnumber")?,
      discs: decoder.number_tag("disctotal")?,
      sample_bits: Some(streaminfo.bits_per_sample.into()),
      sample_rate: streaminfo.sample_rate.into(),
      samples,
      size: (data.len() - Self::frame_offset(data)?).into_u64(),
      title: decoder.text_tag("title")?,
      track: decoder.number_tag("tracknumber")?,
      tracks: decoder.number_tag("tracktotal")?,
    })
  }

  fn frame_offset(data: &[u8]) -> Result<usize, AudioError> {
    let mut offset = 4;

    loop {
      let header = data
        .get(offset..offset + 4)
        .context(audio_error::FlacTruncated)?;

      let length =
        usize::from(header[1]) << 16 | usize::from(header[2]) << 8 | usize::from(header[3]);

      offset += 4 + length;

      ensure!(offset <= data.len(), audio_error::FlacTruncated);

      if header[0] & 0x80 != 0 {
        return Ok(offset);
      }
    }
  }

  fn number_tag(&self, tag: &'static str) -> Result<u64, AudioError> {
    let value = self.tag(tag)?;
    parse_number(value).context(audio_error::TagInteger { tag })
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<AudioMetadata> {
    let data = filesystem::read(path)?;

    FlacDecoder::decode(&data).context(error::Audio { path })
  }

  fn tag(&self, tag: &'static str) -> Result<&str, AudioError> {
    Audio::tag(self.reader.get_tag(tag), tag)
  }

  fn text_tag(&self, tag: &'static str) -> Result<Text, AudioError> {
    self
      .tag(tag)?
      .parse()
      .context(audio_error::TagInvalid { tag })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_err() {
    fn err(bytes: &[u8]) -> AudioError {
      FlacDecoder::decode(bytes).unwrap_err()
    }

    assert_matches!(err(b"foo"), AudioError::FlacDecode { .. });

    assert_matches!(
      err(&flac(&[], 44100)),
      AudioError::TagMissing { tag: "album" },
    );

    assert_matches!(
      err(&flac(&["ALBUM=qux", "TITLE=bar"], 44100)),
      AudioError::TagMissing { tag: "artist" },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ARTIST=baz", "DISCNUMBER=1", "DISCTOTAL=1"],
        44100,
      )),
      AudioError::TagMissing { tag: "title" },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ALBUM=quux", "ARTIST=baz", "TITLE=bar"],
        44100,
      )),
      AudioError::TagMultiple { tag: "album" },
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
      AudioError::TagEmpty { tag: "title" },
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
      AudioError::TagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "title",
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
      AudioError::TagMissing { tag: "tracknumber" },
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
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
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
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
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
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
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
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
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
      AudioError::TagInteger {
        source: NumberError::Integer { .. },
        tag: "tracknumber",
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
      AudioError::FlacSampleCountUnknown,
    );
  }

  #[test]
  fn frame_offset() {
    let bytes = flac(&[], 44100);
    assert_eq!(
      FlacDecoder::frame_offset(&bytes).unwrap(),
      bytes.len() - 1024,
    );

    let bytes = flac(&["foo=bar"], 44100);
    assert_eq!(
      FlacDecoder::frame_offset(&bytes).unwrap(),
      bytes.len() - 1024,
    );

    assert_matches!(
      FlacDecoder::frame_offset(b"fLaC"),
      Err(AudioError::FlacTruncated),
    );

    assert_matches!(
      FlacDecoder::frame_offset(&[b"fLaC".as_slice(), &[0x80, 0x00, 0x00, 0x22]].concat()),
      Err(AudioError::FlacTruncated),
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
        size: 1024,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );
  }
}
