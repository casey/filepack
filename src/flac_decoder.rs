use super::*;

pub(crate) struct FlacDecoder<'a> {
  reader: FlacReader<&'a [u8]>,
}

impl<'a> FlacDecoder<'a> {
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

  fn metadata(data: &'a [u8]) -> Result<AudioMetadata, AudioError> {
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

  fn number_tag(&self, tag: &'static str) -> Result<u64, AudioError> {
    let value = self.tag(tag)?;
    parse_number(value).context(audio_error::TagInteger { tag })
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<AudioMetadata> {
    let data = filesystem::read(path)?;

    FlacDecoder::metadata(&data).context(error::Audio { path })
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
  fn frame_offset() {
    let bytes = FlacBuilder::new().build();
    assert_eq!(
      FlacDecoder::frame_offset(&bytes).unwrap(),
      bytes.len() - 1024,
    );

    let bytes = FlacBuilder::new().tag("foo", "bar").build();
    assert_eq!(
      FlacDecoder::frame_offset(&bytes).unwrap(),
      bytes.len() - 1024,
    );

    assert_matches!(
      FlacDecoder::frame_offset(&FlacBuilder::new().truncate(4).build()),
      Err(AudioError::FlacTruncated),
    );

    assert_matches!(
      FlacDecoder::frame_offset(&FlacBuilder::new().truncate(8).build()),
      Err(AudioError::FlacTruncated),
    );
  }

  #[test]
  fn metadata_err() {
    fn err(builder: FlacBuilder) -> AudioError {
      FlacDecoder::metadata(&builder.build()).unwrap_err()
    }

    assert_matches!(
      FlacDecoder::metadata(b"foo").unwrap_err(),
      AudioError::FlacDecode { .. },
    );

    assert_matches!(
      err(FlacBuilder::new()),
      AudioError::TagMissing { tag: "album" },
    );

    assert_matches!(
      err(FlacBuilder::new().tag("ALBUM", "qux").tag("TITLE", "bar")),
      AudioError::TagMissing { tag: "artist" },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
      ),
      AudioError::TagMissing { tag: "title" },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ALBUM", "quux")
          .tag("ARTIST", "baz")
          .tag("TITLE", "bar")
      ),
      AudioError::TagMultiple { tag: "album" },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "")
      ),
      AudioError::TagEmpty { tag: "title" },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "foo\tbar")
      ),
      AudioError::TagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "title",
      },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
      ),
      AudioError::TagMissing { tag: "tracknumber" },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
          .tag("TRACKNUMBER", "foo")
      ),
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
      },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
          .tag("TRACKNUMBER", "3/12")
      ),
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
      },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
          .tag("TRACKNUMBER", "01")
      ),
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
      },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
          .tag("TRACKNUMBER", "+1")
      ),
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
      },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
          .tag("TRACKNUMBER", "18446744073709551616")
      ),
      AudioError::TagInteger {
        source: NumberError::Integer { .. },
        tag: "tracknumber",
      },
    );

    assert_matches!(
      err(
        FlacBuilder::new()
          .tag("ALBUM", "qux")
          .tag("ARTIST", "baz")
          .tag("DISCNUMBER", "1")
          .tag("DISCTOTAL", "1")
          .tag("TITLE", "bar")
          .tag("TRACKNUMBER", "1")
          .tag("TRACKTOTAL", "1")
          .samples(0)
      ),
      AudioError::FlacSampleCountUnknown,
    );
  }

  #[test]
  fn read_ok() {
    let (_tempdir, root) = tempdir();

    let path = root.join("foo.flac");

    std::fs::write(
      &path,
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "2")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "3")
        .tag("TRACKTOTAL", "4")
        .samples(66150)
        .build(),
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
