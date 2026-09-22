use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
pub(crate) struct Audio {
  #[n(0)]
  pub(crate) channels: u64,
  #[n(1)]
  pub(crate) path: RelativePath,
  #[n(2)]
  pub(crate) sample_bits: Option<u64>,
  #[n(3)]
  pub(crate) sample_rate: u64,
  #[n(4)]
  pub(crate) samples: u64,
  #[n(5)]
  pub(crate) size: u64,
  #[n(6)]
  #[serde(rename = "type")]
  pub(crate) ty: Option<AudioType>,
}

impl Audio {
  pub(crate) fn cover_art(&self, root: &Utf8Path) -> Result<Vec<EmbeddedImage>> {
    let Some(ty) = self.ty else {
      return Ok(Vec::new());
    };

    let path = root.join(&self.path);

    let data = filesystem::read(&path)?;

    match ty {
      AudioType::Flac => FlacDecoder::cover_art(&data),
      AudioType::Mp3 => Mp3Decoder::cover_art(&data),
    }
    .context(error::Audio { path })
  }

  pub(crate) fn duration(&self) -> Duration {
    if self.sample_rate == 0 {
      return Duration::ZERO;
    }

    let subsecond = u128::from(self.samples % self.sample_rate);

    Duration::new(
      self.samples / self.sample_rate,
      u32::try_from(subsecond * 1_000_000_000 / u128::from(self.sample_rate)).unwrap(),
    )
  }

  pub(crate) fn sum_durations(tracks: &[Item<Audio>]) -> Duration {
    tracks.iter().fold(Duration::ZERO, |sum, audio| {
      sum.saturating_add(audio.content.duration())
    })
  }

  pub(crate) fn tag<'a>(
    mut values: impl Iterator<Item = &'a str>,
    tag: &'static str,
  ) -> Result<&'a str, AudioError> {
    let value = values.next().context(audio_error::TagMissing { tag })?;

    ensure! {
      values.next().is_none(),
      audio_error::TagMultiple { tag },
    }

    ensure! {
      !value.is_empty(),
      audio_error::TagEmpty { tag },
    }

    Ok(value)
  }
}

impl Content for Audio {
  const LABEL: &'static str = "Track";

  type Type = AudioType;

  fn info(&self, builder: InfoBuilder) -> InfoBuilder {
    builder
      .value("duration", DisplayDuration(self.duration()))
      .optional_or_unknown("type", self.ty)
      .optional(
        "sample bits",
        self
          .sample_bits
          .map(|sample_bits| format!("{sample_bits}-bit")),
      )
      .value("sample rate", DisplaySampleRate(self.sample_rate))
      .optional(
        "bit rate",
        DisplayBitrate::new(
          u64::try_from(self.duration().as_millis()).unwrap_or(u64::MAX),
          self.size,
        ),
      )
      .value("channels", self.channels)
      .optional(
        "compression",
        self.ty.map(|ty| match ty {
          AudioType::Flac => Compression::Lossless,
          AudioType::Mp3 => Compression::Lossy,
        }),
      )
      .value("samples", self.samples)
  }

  fn load(root: &Utf8Path, path: RelativePath) -> Result<Item<Self>> {
    let (metadata, ty) = AudioMetadata::load(root, &path)?;

    Ok(metadata.into_item(path, ty))
  }

  fn path(&self) -> &RelativePath {
    &self.path
  }

  #[cfg(test)]
  fn test(path: &str) -> Self {
    let path = path.parse::<RelativePath>().unwrap();
    let ty = AudioType::from_path(&path).unwrap();
    Self {
      channels: 2,
      path,
      sample_bits: Some(16),
      sample_rate: 44100,
      samples: 44100,
      size: 1024,
      ty: Some(ty),
    }
  }

  fn ty(&self) -> Option<Self::Type> {
    self.ty
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duration() {
    #[track_caller]
    fn case(samples: u64, sample_rate: u64, expected: Duration) {
      let mut audio = Audio::test("foo.flac");
      audio.sample_rate = sample_rate;
      audio.samples = samples;
      assert_eq!(audio.duration(), expected);
    }

    case(0, 0, Duration::ZERO);
    case(44100, 44100, Duration::from_secs(1));
    case(66150, 44100, Duration::from_millis(1500));
    case(u64::MAX, u64::MAX - 1, Duration::new(1, 0));
  }

  #[test]
  fn info() {
    let mut audio = Audio::test("foo.flac");
    audio.samples = 66150;
    audio.size = 750;

    assert_eq!(
      Content::info(&audio, InfoBuilder::new()).build(),
      InfoBuilder::new()
        .value("duration", "0:01")
        .value("type", "FLAC")
        .value("sample bits", "16-bit")
        .value("sample rate", "44.1 kHz")
        .value("bit rate", "4 kbit/s")
        .value("channels", "2")
        .value("compression", "lossless")
        .value("samples", "66150")
        .build(),
    );

    let mut audio = Audio::test("foo.mp3");
    audio.sample_bits = None;
    audio.samples = 66150;
    audio.size = 750;

    assert_eq!(
      Content::info(&audio, InfoBuilder::new()).build(),
      InfoBuilder::new()
        .value("duration", "0:01")
        .value("type", "MP3")
        .value("sample rate", "44.1 kHz")
        .value("bit rate", "4 kbit/s")
        .value("channels", "2")
        .value("compression", "lossy")
        .value("samples", "66150")
        .build(),
    );

    let mut audio = Audio::test("foo.flac");
    audio.sample_bits = None;
    audio.sample_rate = 0;
    audio.samples = 0;
    audio.size = 750;

    assert_eq!(
      Content::info(&audio, InfoBuilder::new()).build(),
      InfoBuilder::new()
        .value("duration", "0:00")
        .value("type", "FLAC")
        .value("sample rate", "0 kHz")
        .value("channels", "2")
        .value("compression", "lossless")
        .value("samples", "0")
        .build(),
    );
  }

  #[test]
  fn load() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.flac"),
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

    std::fs::write(
      root.join("foo.mp3"),
      Mp3Builder::new()
        .tag("TALB", "qux")
        .tag("TIT2", "bar")
        .tag("TPE1", "baz")
        .tag("TPOS", "1/2")
        .tag("TRCK", "3/4")
        .frames(2)
        .build(),
    )
    .unwrap();

    assert_eq!(
      Audio::load(&root, "foo.flac".parse().unwrap()).unwrap(),
      Item {
        content: Audio {
          channels: 2,
          path: "foo.flac".parse().unwrap(),
          sample_bits: Some(16),
          sample_rate: 44100,
          samples: 66150,
          size: 1024,
          ty: Some(AudioType::Flac),
        },
        title: Some("bar".parse().unwrap()),
      },
    );

    assert_eq!(
      Audio::load(&root, "foo.mp3".parse().unwrap()).unwrap(),
      Item {
        content: Audio {
          channels: 2,
          path: "foo.mp3".parse().unwrap(),
          sample_bits: None,
          sample_rate: 44100,
          samples: 2304,
          size: 834,
          ty: Some(AudioType::Mp3),
        },
        title: Some("bar".parse().unwrap()),
      },
    );
  }

  #[test]
  fn load_rejects_invalid_extension() {
    #[track_caller]
    fn case(path: &str, expected: &str) {
      let (_tempdir, root) = tempdir();

      assert_eq!(
        Audio::load(&root, path.parse().unwrap())
          .unwrap_err()
          .iter_chain()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(": "),
        expected,
      );
    }

    case(
      "foo.wav",
      "invalid path `foo.wav`: path must end in `.flac` or `.mp3`",
    );
    case(
      "foo",
      "invalid path `foo`: path must end in `.flac` or `.mp3`",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Audio {
        channels: 8,
        path: "foo.flac".parse().unwrap(),
        sample_bits: Some(7),
        sample_rate: 1,
        samples: 2,
        size: 9,
        ty: Some(AudioType::Flac),
      })
      .unwrap(),
      r#"{"channels":8,"path":"foo.flac","sample_bits":7,"sample_rate":1,"samples":2,"size":9,"type":"flac"}"#,
    );
  }
}
