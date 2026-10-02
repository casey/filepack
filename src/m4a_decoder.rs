use super::*;

#[derive(Clone, Debug, PartialEq)]
struct AudioProperties {
  channels: u64,
  sample_rate: u64,
  samples: u64,
  size: u64,
}

struct Atom<'a> {
  body: &'a [u8],
  ty: [u8; 4],
}

struct Atoms<'a> {
  data: &'a [u8],
  done: bool,
  offset: usize,
}

impl<'a> Atoms<'a> {
  fn new(data: &'a [u8]) -> Self {
    Self {
      data,
      done: false,
      offset: 0,
    }
  }

  fn parse(&mut self) -> Result<Atom<'a>, M4aError> {
    let header = self
      .data
      .get(self.offset..self.offset + 8)
      .context(m4a_error::Truncated)?;

    let size = u32::from_be_bytes(header[..4].try_into().unwrap());

    let ty = header[4..8].try_into().unwrap();

    let (start, end) = match size {
      0 => (self.offset + 8, self.data.len()),
      1 => {
        let size = self
          .data
          .get(self.offset + 8..self.offset + 16)
          .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap()))
          .context(m4a_error::Truncated)?;

        ensure!(size >= 16, m4a_error::BoxSize { size });

        let end = usize::try_from(size)
          .ok()
          .and_then(|size| self.offset.checked_add(size))
          .context(m4a_error::Truncated)?;

        (self.offset + 16, end)
      }
      size => {
        ensure!(size >= 8, m4a_error::BoxSize { size });
        (
          self.offset + 8,
          self.offset + usize::try_from(size).unwrap(),
        )
      }
    };

    let body = self.data.get(start..end).context(m4a_error::Truncated)?;

    self.offset = end;

    Ok(Atom { body, ty })
  }
}

impl<'a> Iterator for Atoms<'a> {
  type Item = Result<Atom<'a>, M4aError>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.done || self.offset >= self.data.len() {
      return None;
    }

    let atom = self.parse();

    if atom.is_err() {
      self.done = true;
    }

    Some(atom)
  }
}

pub(crate) struct M4aDecoder;

impl M4aDecoder {
  fn bytes<const N: usize>(
    body: &[u8],
    offset: usize,
    ty: &'static str,
  ) -> Result<[u8; N], M4aError> {
    body
      .get(offset..offset + N)
      .map(|bytes| bytes.try_into().unwrap())
      .context(m4a_error::BoxInvalid { ty })
  }

  fn child(data: &[u8], ty: [u8; 4]) -> Result<Option<&[u8]>, M4aError> {
    for atom in Atoms::new(data) {
      let atom = atom?;
      if atom.ty == ty {
        return Ok(Some(atom.body));
      }
    }

    Ok(None)
  }

  pub(crate) fn cover_art(data: &[u8]) -> Result<Vec<EmbeddedImage>, AudioError> {
    fn cover_art(data: &[u8]) -> Result<Vec<EmbeddedImage>, M4aError> {
      let moov = M4aDecoder::required(data, "moov")?;

      let ilst = M4aDecoder::ilst(moov)?;

      M4aDecoder::values(ilst, *b"covr")?
        .into_iter()
        .map(|(data_type, data)| {
          let media_type = match data_type {
            13 => mime::IMAGE_JPEG,
            14 => mime::IMAGE_PNG,
            _ => return Err(M4aError::CoverDataType { data_type }),
          };

          Ok(EmbeddedImage {
            data: data.into(),
            media_type,
          })
        })
        .collect()
    }

    cover_art(data).context(audio_error::M4aDecode)
  }

  fn descriptor(data: &[u8], tag: u8) -> Result<&[u8], M4aError> {
    let ty = "esds";

    ensure!(data.first() == Some(&tag), m4a_error::BoxInvalid { ty });

    let mut length = 0;
    let mut offset = 1;

    loop {
      let byte = *data.get(offset).context(m4a_error::BoxInvalid { ty })?;
      offset += 1;
      length = length << 7 | usize::from(byte & 0x7f);
      if byte & 0x80 == 0 {
        break;
      }
      ensure!(offset <= 4, m4a_error::BoxInvalid { ty });
    }

    data
      .get(offset..offset + length)
      .context(m4a_error::BoxInvalid { ty })
  }

  fn fourcc(fourcc: [u8; 4]) -> String {
    str::from_utf8(&fourcc).map_or_else(|_| "unknown".into(), Into::into)
  }

  fn full<'a>(body: &'a [u8], ty: &'static str) -> Result<(u8, &'a [u8]), M4aError> {
    ensure!(body.len() >= 4, m4a_error::BoxInvalid { ty });
    Ok((body[0], &body[4..]))
  }

  fn ilst(moov: &[u8]) -> Result<Option<&[u8]>, M4aError> {
    let meta = match Self::child(moov, *b"udta")? {
      Some(udta) => Self::child(udta, *b"meta")?,
      None => None,
    };

    let meta = match meta {
      Some(meta) => Some(meta),
      None => Self::child(moov, *b"meta")?,
    };

    let Some(meta) = meta else {
      return Ok(None);
    };

    let (_version, meta) = Self::full(meta, "meta")?;

    Self::child(meta, *b"ilst")
  }

  fn metadata(data: &[u8]) -> Result<AudioMetadata, AudioError> {
    let moov = Self::required(data, "moov").context(audio_error::M4aDecode)?;

    let AudioProperties {
      channels,
      sample_rate,
      samples,
      size,
    } = Self::properties(moov).context(audio_error::M4aDecode)?;

    let ilst = Self::ilst(moov).context(audio_error::M4aDecode)?;

    let album = Self::text_tag(ilst, *b"\xa9alb", "©alb")?;
    let artist = Self::text_tag(ilst, *b"\xa9ART", "©ART")?;
    let (disc, discs) = Self::pair_tag(ilst, *b"disk", "disk")?;
    let discs = discs.context(audio_error::DiscTotalMissing { tag: "disk" })?;
    let title = Self::text_tag(ilst, *b"\xa9nam", "©nam")?;
    let (track, tracks) = Self::pair_tag(ilst, *b"trkn", "trkn")?;
    let tracks = tracks.context(audio_error::TrackTotalMissing { tag: "trkn" })?;

    Ok(AudioMetadata {
      album,
      artist,
      channels,
      codec: AudioCodec::Aac,
      disc,
      discs,
      sample_bits: None,
      sample_rate,
      samples,
      size,
      title,
      track,
      tracks,
    })
  }

  fn object_type(esds: &[u8]) -> Result<u8, M4aError> {
    let ty = "esds";

    let es = Self::descriptor(esds, 0x03)?;

    let flags = *es.get(2).context(m4a_error::BoxInvalid { ty })?;

    let mut offset = 3;

    if flags & 0x80 != 0 {
      offset += 2;
    }

    if flags & 0x40 != 0 {
      offset += 1 + usize::from(*es.get(offset).context(m4a_error::BoxInvalid { ty })?);
    }

    if flags & 0x20 != 0 {
      offset += 2;
    }

    let config = Self::descriptor(
      es.get(offset..).context(m4a_error::BoxInvalid { ty })?,
      0x04,
    )?;

    config
      .first()
      .copied()
      .context(m4a_error::BoxInvalid { ty })
  }

  fn pair_tag(
    ilst: Option<&[u8]>,
    fourcc: [u8; 4],
    tag: &'static str,
  ) -> Result<(u64, Option<u64>), AudioError> {
    fn pair(value: &[u8], tag: &'static str) -> Result<(u64, Option<u64>), M4aError> {
      let bytes = value.get(2..6).context(m4a_error::PairTag { tag })?;
      let number = u16::from_be_bytes(bytes[..2].try_into().unwrap());
      let total = u16::from_be_bytes(bytes[2..].try_into().unwrap());
      Ok((number.into(), (total != 0).then_some(total.into())))
    }

    let mut values = Self::values(ilst, fourcc)
      .context(audio_error::M4aDecode)?
      .into_iter();

    let (data_type, value) = values.next().context(audio_error::TagMissing { tag })?;

    ensure!(values.next().is_none(), audio_error::TagMultiple { tag });

    if data_type != 0 {
      return Err(M4aError::DataType { data_type, tag }).context(audio_error::M4aDecode);
    }

    pair(value, tag).context(audio_error::M4aDecode)
  }

  fn properties(moov: &[u8]) -> Result<AudioProperties, M4aError> {
    let mut audio = None;
    let mut index = 0;

    for trak in Atoms::new(moov) {
      let trak = trak?;

      if trak.ty != *b"trak" {
        continue;
      }

      let mdia = Self::required(trak.body, "mdia")?;

      let (_version, hdlr) = Self::full(Self::required(mdia, "hdlr")?, "hdlr")?;

      match &Self::bytes::<4>(hdlr, 4, "hdlr")? {
        b"soun" => {
          ensure!(audio.is_none(), m4a_error::AudioTrackMultiple);
          audio = Some(Self::track(mdia)?);
        }
        ty => {
          return Err(M4aError::TrackUnsupported {
            track: index,
            ty: match ty {
              b"auxv" => "auxiliary video",
              b"meta" => "metadata",
              b"pict" => "picture",
              b"vide" => "video",
              _ => "unknown",
            },
          });
        }
      }

      index += 1;
    }

    audio.context(m4a_error::AudioTrackMissing)
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<AudioMetadata> {
    let data = filesystem::read(path)?;

    Self::metadata(&data).context(error::Audio { path })
  }

  fn required<'a>(data: &'a [u8], ty: &'static str) -> Result<&'a [u8], M4aError> {
    Self::child(data, ty.as_bytes().try_into().unwrap())?.context(m4a_error::BoxMissing { ty })
  }

  fn text_tag(ilst: Option<&[u8]>, fourcc: [u8; 4], tag: &'static str) -> Result<Text, AudioError> {
    let values = Self::values(ilst, fourcc).context(audio_error::M4aDecode)?;

    let mut strings = Vec::new();

    for (data_type, value) in values {
      if data_type != 1 {
        return Err(M4aError::DataType { data_type, tag }).context(audio_error::M4aDecode);
      }

      strings.push(str::from_utf8(value).context(audio_error::TagUtf8 { tag })?);
    }

    Audio::tag(strings.into_iter(), tag)?
      .parse()
      .context(audio_error::TagInvalid { tag })
  }

  fn track(mdia: &[u8]) -> Result<AudioProperties, M4aError> {
    let (version, mdhd) = Self::full(Self::required(mdia, "mdhd")?, "mdhd")?;

    let timescale = match version {
      0 => u32::from_be_bytes(Self::bytes(mdhd, 8, "mdhd")?),
      1 => u32::from_be_bytes(Self::bytes(mdhd, 16, "mdhd")?),
      _ => return Err(M4aError::BoxInvalid { ty: "mdhd" }),
    };

    ensure!(timescale != 0, m4a_error::TimescaleZero);

    let stbl = Self::required(Self::required(mdia, "minf")?, "stbl")?;

    let (_version, stsd) = Self::full(Self::required(stbl, "stsd")?, "stsd")?;

    let entries = stsd
      .get(4..)
      .context(m4a_error::BoxInvalid { ty: "stsd" })?;

    let entry = Atoms::new(entries)
      .next()
      .context(m4a_error::BoxInvalid { ty: "stsd" })??;

    ensure! {
      entry.ty == *b"mp4a",
      m4a_error::CodecUnsupported { codec: Self::fourcc(entry.ty) },
    }

    let mp4a = entry.body;

    let version = u16::from_be_bytes(Self::bytes(mp4a, 8, "mp4a")?);
    let channels = u16::from_be_bytes(Self::bytes(mp4a, 16, "mp4a")?);
    let sample_rate = u32::from_be_bytes(Self::bytes(mp4a, 24, "mp4a")?) >> 16;

    let children = match version {
      0 => 28,
      1 => 44,
      _ => return Err(M4aError::BoxInvalid { ty: "mp4a" }),
    };

    let children = mp4a
      .get(children..)
      .context(m4a_error::BoxInvalid { ty: "mp4a" })?;

    let (_version, esds) = Self::full(Self::required(children, "esds")?, "esds")?;

    match Self::object_type(esds)? {
      0x40 | 0x66 | 0x67 => {}
      0x69 | 0x6b => {
        return Err(M4aError::CodecUnsupported {
          codec: "MP3".into(),
        });
      }
      _ => {
        return Err(M4aError::CodecUnsupported {
          codec: "unknown".into(),
        });
      }
    }

    let (_version, stts) = Self::full(Self::required(stbl, "stts")?, "stts")?;

    let entries = u32::from_be_bytes(Self::bytes(stts, 0, "stts")?);

    let mut duration = 0u128;

    for i in 0..usize::try_from(entries).unwrap() {
      let count = u32::from_be_bytes(Self::bytes(stts, 4 + i * 8, "stts")?);
      let delta = u32::from_be_bytes(Self::bytes(stts, 8 + i * 8, "stts")?);
      duration += u128::from(count) * u128::from(delta);
    }

    let samples = u64::try_from(duration * u128::from(sample_rate) / u128::from(timescale))
      .ok()
      .context(m4a_error::SamplesOverflow)?;

    ensure!(samples > 0, m4a_error::Empty);

    let (_version, stsz) = Self::full(Self::required(stbl, "stsz")?, "stsz")?;

    let sample_size = u32::from_be_bytes(Self::bytes(stsz, 0, "stsz")?);
    let sample_count = u32::from_be_bytes(Self::bytes(stsz, 4, "stsz")?);

    let size = if sample_size == 0 {
      let mut size = 0;
      for i in 0..usize::try_from(sample_count).unwrap() {
        size += u64::from(u32::from_be_bytes(Self::bytes(stsz, 8 + i * 4, "stsz")?));
      }
      size
    } else {
      u64::from(sample_size) * u64::from(sample_count)
    };

    Ok(AudioProperties {
      channels: channels.into(),
      sample_rate: sample_rate.into(),
      samples,
      size,
    })
  }

  fn values(ilst: Option<&[u8]>, fourcc: [u8; 4]) -> Result<Vec<(u32, &[u8])>, M4aError> {
    let mut values = Vec::new();

    let Some(ilst) = ilst else {
      return Ok(values);
    };

    for item in Atoms::new(ilst) {
      let item = item?;

      if item.ty != fourcc {
        continue;
      }

      for data in Atoms::new(item.body) {
        let data = data?;

        if data.ty != *b"data" {
          continue;
        }

        let header = Self::bytes::<8>(data.body, 0, "data")?;

        let data_type = u32::from_be_bytes(header[..4].try_into().unwrap()) & 0x00ff_ffff;

        values.push((data_type, &data.body[8..]));
      }
    }

    Ok(values)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cover_art() {
    #[track_caller]
    fn case(builder: Mp4Builder, expected: &[(&[u8], Mime)]) {
      assert_eq!(
        M4aDecoder::cover_art(&builder.audio_track(0x40).build()).unwrap(),
        expected
          .iter()
          .map(|(data, media_type)| EmbeddedImage {
            data: data.to_vec(),
            media_type: media_type.clone(),
          })
          .collect::<Vec<EmbeddedImage>>(),
      );
    }

    case(Mp4Builder::new(), &[]);
    case(Mp4Builder::new().name("foo"), &[]);
    case(
      Mp4Builder::new().picture(13, b"foo"),
      &[(b"foo", mime::IMAGE_JPEG)],
    );
    case(
      Mp4Builder::new().picture(14, b"foo"),
      &[(b"foo", mime::IMAGE_PNG)],
    );
    case(
      Mp4Builder::new().picture(14, b"foo").picture(13, b"bar"),
      &[(b"foo", mime::IMAGE_PNG), (b"bar", mime::IMAGE_JPEG)],
    );

    assert_matches!(
      M4aDecoder::cover_art(&Mp4Builder::new().picture(0, b"foo").build()).unwrap_err(),
      AudioError::M4aDecode {
        source: M4aError::CoverDataType { data_type: 0 },
      },
    );

    assert_matches!(
      M4aDecoder::cover_art(b"foo").unwrap_err(),
      AudioError::M4aDecode {
        source: M4aError::Truncated,
      },
    );
  }

  fn m4a() -> Mp4Builder {
    Mp4Builder::new()
      .tag(*b"\xa9alb", "qux")
      .tag(*b"\xa9ART", "baz")
      .tag(*b"\xa9nam", "bar")
      .pair_tag(*b"disk", 1, 2)
      .pair_tag(*b"trkn", 3, 4)
      .frame_count(2)
  }

  #[test]
  fn metadata_err() {
    #[track_caller]
    fn case(builder: Mp4Builder, expected: M4aError) {
      match M4aDecoder::metadata(&builder.build()).unwrap_err() {
        AudioError::M4aDecode { source } => assert_eq!(source, expected),
        err => panic!("unexpected error: {err}"),
      }
    }

    fn err(builder: Mp4Builder) -> AudioError {
      M4aDecoder::metadata(&builder.build()).unwrap_err()
    }

    assert_matches!(
      M4aDecoder::metadata(b"foo").unwrap_err(),
      AudioError::M4aDecode {
        source: M4aError::Truncated,
      },
    );

    case(Mp4Builder::new(), M4aError::AudioTrackMissing);

    case(
      m4a().audio_track(0x40).audio_track(0x40),
      M4aError::AudioTrackMultiple,
    );

    case(
      m4a().video_track(2, 1).audio_track(0x40),
      M4aError::TrackUnsupported {
        track: 0,
        ty: "video",
      },
    );

    case(
      m4a().audio_track(0x40).track(*b"meta", 1000, &[]),
      M4aError::TrackUnsupported {
        track: 1,
        ty: "metadata",
      },
    );

    case(
      m4a().audio_track(0x69),
      M4aError::CodecUnsupported {
        codec: "MP3".into(),
      },
    );

    case(
      m4a().audio_track(0x6b),
      M4aError::CodecUnsupported {
        codec: "MP3".into(),
      },
    );

    case(
      m4a().audio_track(0x11),
      M4aError::CodecUnsupported {
        codec: "unknown".into(),
      },
    );

    case(
      m4a().track(
        *b"soun",
        44100,
        &[Mp4Builder::video_entry(*b"fLaC", *b"dfLa", &[], 0, 0)],
      ),
      M4aError::CodecUnsupported {
        codec: "fLaC".into(),
      },
    );

    case(
      m4a().track(*b"soun", 44100, &[]),
      M4aError::BoxInvalid { ty: "stsd" },
    );

    case(
      m4a().mp4a_version(2).audio_track(0x40),
      M4aError::BoxInvalid { ty: "mp4a" },
    );

    case(
      m4a().media_timescale(0).audio_track(0x40),
      M4aError::TimescaleZero,
    );

    case(m4a().frame_count(0).audio_track(0x40), M4aError::Empty);

    case(
      m4a().data(*b"\xa9alb", 0, b"qux").audio_track(0x40),
      M4aError::DataType {
        data_type: 0,
        tag: "©alb",
      },
    );

    case(
      Mp4Builder::new()
        .tag(*b"\xa9alb", "qux")
        .tag(*b"\xa9ART", "baz")
        .pair_tag(*b"disk", 1, 2)
        .tag(*b"\xa9nam", "bar")
        .data(*b"trkn", 1, b"qux")
        .frame_count(2)
        .audio_track(0x40),
      M4aError::DataType {
        data_type: 1,
        tag: "trkn",
      },
    );

    case(
      Mp4Builder::new()
        .tag(*b"\xa9alb", "qux")
        .tag(*b"\xa9ART", "baz")
        .data(*b"disk", 0, &[0; 4])
        .frame_count(2)
        .audio_track(0x40),
      M4aError::PairTag { tag: "disk" },
    );

    assert_matches!(
      err(Mp4Builder::new().frame_count(2).audio_track(0x40)),
      AudioError::TagMissing { tag: "©alb" },
    );

    assert_matches!(
      err(m4a().tag(*b"\xa9alb", "quux").audio_track(0x40)),
      AudioError::TagMultiple { tag: "©alb" },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "")
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagEmpty { tag: "©alb" },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "qux")
          .tag(*b"\xa9ART", "baz")
          .tag(*b"\xa9nam", "foo\tbar")
          .pair_tag(*b"disk", 1, 2)
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "©nam",
      },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", b"\xff")
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagUtf8 { tag: "©alb", .. },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "qux")
          .tag(*b"\xa9ART", "baz")
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagMissing { tag: "disk" },
    );

    assert_matches!(
      err(m4a().pair_tag(*b"trkn", 3, 4).audio_track(0x40)),
      AudioError::TagMultiple { tag: "trkn" },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "qux")
          .tag(*b"\xa9ART", "baz")
          .tag(*b"\xa9nam", "bar")
          .pair_tag(*b"disk", 1, 1)
          .pair_tag(*b"trkn", 1, 0)
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TrackTotalMissing { tag: "trkn" },
    );
  }

  #[test]
  fn metadata_ok() {
    #[track_caller]
    fn case(builder: Mp4Builder, expected: AudioProperties) {
      let AudioMetadata {
        channels,
        sample_rate,
        samples,
        size,
        ..
      } = M4aDecoder::metadata(&builder.build()).unwrap();

      assert_eq!(
        AudioProperties {
          channels,
          sample_rate,
          samples,
          size,
        },
        expected,
      );
    }

    assert_eq!(
      M4aDecoder::metadata(&m4a().audio_track(0x40).build()).unwrap(),
      AudioMetadata {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        codec: AudioCodec::Aac,
        disc: 1,
        discs: 2,
        sample_bits: None,
        sample_rate: 44100,
        samples: 2,
        size: 2,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );

    let expected = AudioProperties {
      channels: 2,
      sample_rate: 44100,
      samples: 2,
      size: 2,
    };

    case(m4a().audio_track(0x66), expected.clone());
    case(m4a().audio_track(0x67), expected.clone());
    case(
      m4a().esds_long_lengths().audio_track(0x40),
      expected.clone(),
    );
    case(m4a().mp4a_version(1).audio_track(0x40), expected.clone());
    case(m4a().meta_in_moov().audio_track(0x40), expected);

    case(
      m4a().sample_size(5).audio_track(0x40),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 2,
        size: 10,
      },
    );

    case(
      m4a().sample_sizes(&[3, 5]).audio_track(0x40),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 2,
        size: 8,
      },
    );

    case(
      m4a().media_timescale(22050).audio_track(0x40),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 4,
        size: 2,
      },
    );
  }

  #[test]
  fn read_ok() {
    let (_tempdir, root) = tempdir();

    let path = root.join("foo.m4a");

    std::fs::write(&path, m4a().audio_track(0x40).build()).unwrap();

    assert_eq!(
      M4aDecoder::read(&path).unwrap(),
      AudioMetadata {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        codec: AudioCodec::Aac,
        disc: 1,
        discs: 2,
        sample_bits: None,
        sample_rate: 44100,
        samples: 2,
        size: 2,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );
  }
}
