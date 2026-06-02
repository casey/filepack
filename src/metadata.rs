use super::*;

#[allow(private_interfaces)]
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Encode, Decode, PartialEq, Serialize)]
pub struct Metadata {
  #[n(0)]
  pub artwork: Option<filename::Artwork>,
  #[n(1)]
  pub creator: Option<ComponentBuf>,
  #[n(2)]
  pub date: Option<DateTime>,
  #[n(3)]
  pub description: Option<String>,
  #[n(4)]
  pub homepage: Option<CheckedUrl>,
  #[n(5)]
  pub language: Option<Language>,
  #[n(6)]
  pub media: Option<Media>,
  #[n(7)]
  pub package: Option<Package>,
  #[n(8)]
  pub readme: Option<filename::Md>,
  #[n(9)]
  pub title: Option<ComponentBuf>,
}

impl Metadata {
  pub(crate) const CBOR_FILENAME: &'static str = "metadata.filepack";
  pub(crate) const YAML_FILENAME: &'static str = "metadata.yaml";

  pub(crate) fn check(&self, root: &Utf8Path, paths: &HashSet<RelativePath>) -> Result {
    for filename in self.files() {
      ensure! {
        paths.contains(&filename),
        error::MissingMetadataFile { filename },
      }
    }

    if let Some(artwork) = &self.artwork {
      Self::check_artwork(root, artwork)?;
    }

    Ok(())
  }

  fn check_artwork(root: &Utf8Path, artwork: &filename::Artwork) -> Result {
    let path = root.join(artwork.as_path());

    let dimensions = match artwork.ty() {
      ArtworkType::Jpeg => Self::decode_jpeg(&path)?,
      ArtworkType::Png => Self::decode_png(&path)?,
    };

    ensure! {
      dimensions.width == dimensions.height,
      error::ArtworkDimensions {
        dimensions,
        path,
      }
    }

    Ok(())
  }

  fn decode_jpeg(path: &Utf8Path) -> Result<Dimensions> {
    let bytes = filesystem::read(path)?;

    let mut decoder = JpegDecoder::new(io::Cursor::new(bytes));

    decoder
      .decode_headers()
      .context(error::ArtworkDecodeJpeg { path })?;

    let info = decoder.info().unwrap();

    Ok(Dimensions {
      height: info.height.into(),
      width: info.width.into(),
    })
  }

  fn decode_png(path: &Utf8Path) -> Result<Dimensions> {
    let bytes = filesystem::read(path)?;

    let reader = png::Decoder::new(io::Cursor::new(bytes))
      .read_info()
      .context(error::ArtworkDecodePng { path })?;

    let info = reader.info();

    Ok(Dimensions {
      height: info.height,
      width: info.width,
    })
  }

  pub(crate) fn deserialize(path: &Utf8Path, yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml).context(error::DeserializeMetadata { path })
  }

  pub(crate) fn deserialize_strict(path: &Utf8Path, yaml: &str) -> Result<Self> {
    let deserializer = serde_yaml::Deserializer::from_str(yaml);

    let mut unknown = BTreeSet::new();

    let metadata = serde_ignored::deserialize(deserializer, |path| {
      unknown.insert(path.to_string());
    })
    .context(error::DeserializeMetadata { path })?;

    if !unknown.is_empty() {
      return Err(error::DeserializeMetadataStrict { path, unknown }.build());
    }

    Ok(metadata)
  }

  pub(crate) fn files(&self) -> Vec<RelativePath> {
    let mut files = Vec::new();

    if let Some(artwork) = &self.artwork {
      files.push(artwork.as_path());
    }

    if let Some(package) = &self.package
      && let Some(nfo) = &package.nfo
    {
      files.push(nfo.as_path());
    }

    if let Some(readme) = &self.readme {
      files.push(readme.as_path());
    }

    if let Some(media) = &self.media {
      match media {
        Media::Audio { tracks } => files.extend(tracks.iter().map(Filename::as_path)),
      }
    }

    files
  }
}

#[cfg(test)]
mod tests {
  use {super::*, image::ImageFormat};

  const UNKNOWN_FIELD: &str = "title: foo\nbar: 1";

  #[test]
  fn deserialize_allows_missing_optional_fields() {
    Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), "title: Foo").unwrap();
  }

  #[test]
  fn deserialize_media_audio() {
    let metadata = Metadata::deserialize(
      Metadata::YAML_FILENAME.as_ref(),
      "media:\n  type: audio\n  tracks:\n    - foo.flac\n    - bar.flac\n",
    )
    .unwrap();

    assert_eq!(
      metadata.media,
      Some(Media::Audio {
        tracks: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
      }),
    );
  }

  #[test]
  fn deserialize_rejects_invalid_values() {
    #[track_caller]
    fn case(yaml: &str, expected: &str) {
      let error = Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), yaml).unwrap_err();

      let chain = error
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": ");

      assert_matches_regex!(chain, expected);
    }

    case(
      "title: Foo\ndate: 2024/06/15",
      "date: input contains invalid characters",
    );
    case(
      "title: Foo\nhomepage: not-a-valid-url",
      "homepage: relative URL without a base",
    );
    case("title: Foo\nlanguage: ac", "unknown language code `ac`");
    case(
      "title: Foo\npackage:\n  creator_tag: foo",
      r"package\.creator_tag: tags must match regex",
    );
    case(
      "title: Foo\npackage:\n  date: not-a-date",
      r"package\.date: input contains invalid characters",
    );
    case(
      "title: Foo\npackage:\n  homepage: :::invalid",
      "package.homepage: relative URL without a base",
    );
    case(
      "title: Foo\nartwork: cover.svg",
      "artwork: component must end in `.jpg` or `.png`",
    );
    case(
      "title: Foo\npackage:\n  nfo: info.txt",
      "nfo: component must end in `.nfo`",
    );
    case(
      "title: Foo\nreadme: README.txt",
      "readme: component must end in `.md`",
    );
    case(
      "title: Foo\nmedia:\n  type: audio\n  tracks:\n    - foo.mp3",
      r"component must end in `\.flac`",
    );
  }

  #[test]
  fn deserializer_allows_unknown_fields() {
    Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), UNKNOWN_FIELD).unwrap();
  }

  #[test]
  fn encoding() {
    assert_encoding(Metadata {
      artwork: Some("cover.png".parse().unwrap()),
      creator: Some("foo".parse().unwrap()),
      date: Some("2024".parse().unwrap()),
      description: Some("bar".into()),
      homepage: Some("http://example.com".parse().unwrap()),
      language: Some("en".parse().unwrap()),
      media: Some(Media::Audio {
        tracks: vec!["track.flac".parse().unwrap()],
      }),
      package: Some(Package {
        creator: Some("baz".parse().unwrap()),
        creator_tag: Some("A0".parse().unwrap()),
        date: Some("2024-01-01".parse().unwrap()),
        description: Some("qux".into()),
        homepage: Some("http://example.com/foo".parse().unwrap()),
        nfo: Some("info.nfo".parse().unwrap()),
        title: Some("foo-bar".parse().unwrap()),
      }),
      readme: Some("README.md".parse().unwrap()),
      title: Some("foo".parse().unwrap()),
    });
  }

  #[test]
  fn filepack_metadata_is_valid() {
    Metadata::deserialize_strict(
      Metadata::YAML_FILENAME.as_ref(),
      &filesystem::read_to_string(Metadata::YAML_FILENAME).unwrap(),
    )
    .unwrap();
  }

  #[test]
  fn files_includes_audio_tracks() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        tracks: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
      }),
      ..default()
    };

    assert_eq!(
      metadata.files(),
      vec![
        "foo.flac".parse::<RelativePath>().unwrap(),
        "bar.flac".parse().unwrap(),
      ],
    );
  }

  fn image(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
    let mut buffer = io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(width, height)
      .write_to(&mut buffer, image_format)
      .unwrap();
    buffer.into_inner()
  }

  #[test]
  fn invalid_artwork() {
    #[track_caller]
    fn case(filename: &str, bytes: Vec<u8>, expected: &str) {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(filename), bytes).unwrap();

      let metadata = Metadata {
        artwork: Some(filename.parse().unwrap()),
        ..default()
      };

      let paths = HashSet::from([filename.parse::<RelativePath>().unwrap()]);

      assert_matches_regex!(
        metadata.check(&root, &paths).unwrap_err().to_string(),
        expected
      );
    }

    case(
      "cover.jpg",
      b"bar".to_vec(),
      "failed to decode JPEG artwork `.*cover\\.jpg`",
    );
    case(
      "cover.png",
      b"bar".to_vec(),
      "failed to decode PNG artwork `.*cover\\.png`",
    );
    case(
      "cover.jpg",
      image(1, 1, ImageFormat::Png),
      "failed to decode JPEG artwork `.*cover\\.jpg`",
    );
    case(
      "cover.png",
      image(1, 1, ImageFormat::Jpeg),
      "failed to decode PNG artwork `.*cover\\.png`",
    );
    case(
      "cover.jpg",
      image(2, 1, ImageFormat::Jpeg),
      "^artwork `.*cover\\.jpg` is 2×1 but must be square$",
    );
    case(
      "cover.png",
      image(2, 1, ImageFormat::Png),
      "^artwork `.*cover\\.png` is 2×1 but must be square$",
    );
  }

  #[test]
  fn metadata_in_readme_is_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```yaml(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let metadata = Metadata::deserialize_strict("README.md".as_ref(), &capture[1]).unwrap();

      let Metadata {
        artwork,
        creator,
        date,
        description,
        homepage,
        language,
        package,
        readme,
        title,
        media,
      } = metadata;

      if title
        .as_ref()
        .is_none_or(|title| *title != "Tobin's Spirit Guide")
      {
        continue;
      }

      assert!(artwork.is_some());
      assert!(creator.is_some());
      assert!(date.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(language.is_some());
      assert!(readme.is_some());
      assert!(title.is_some());
      assert!(media.is_none());

      let Package {
        creator,
        creator_tag,
        date,
        description,
        homepage,
        nfo,
        title,
      } = package.unwrap();

      assert!(creator.is_some());
      assert!(creator_tag.is_some());
      assert!(date.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(nfo.is_some());
      assert!(title.is_some());
    }
  }

  #[test]
  fn missing_files() {
    #[track_caller]
    fn case(metadata: Metadata, filename: &str) {
      assert_eq!(
        metadata
          .check(Utf8Path::new(""), &HashSet::new())
          .unwrap_err()
          .to_string(),
        format!("file referenced in metadata missing: `{filename}`"),
      );
    }

    case(
      Metadata {
        artwork: Some("cover.png".parse().unwrap()),
        ..default()
      },
      "cover.png",
    );

    case(
      Metadata {
        readme: Some("README.md".parse().unwrap()),
        ..default()
      },
      "README.md",
    );

    case(
      Metadata {
        package: Some(nfo_package("info.nfo")),
        ..default()
      },
      "info.nfo",
    );
  }

  fn nfo_package(nfo: &str) -> Package {
    Package {
      creator: None,
      creator_tag: None,
      date: None,
      description: None,
      homepage: None,
      nfo: Some(nfo.parse().unwrap()),
      title: None,
    }
  }

  #[test]
  fn strict_deserialize_rejects_unknown_fields() {
    assert_eq!(
      Metadata::deserialize_strict(Metadata::YAML_FILENAME.as_ref(), UNKNOWN_FIELD)
        .unwrap_err()
        .to_string(),
      "unknown fields in metadata at `metadata.yaml`: `bar`",
    );
  }

  #[test]
  fn valid_artwork() {
    #[track_caller]
    fn case(artwork: &str, bytes: Vec<u8>) {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(artwork), bytes).unwrap();

      let metadata = Metadata {
        artwork: Some(artwork.parse().unwrap()),
        package: Some(nfo_package("info.nfo")),
        readme: Some("README.md".parse().unwrap()),
        ..default()
      };

      let paths = [artwork, "info.nfo", "README.md"]
        .into_iter()
        .map(|path| path.parse::<RelativePath>().unwrap())
        .collect();

      metadata.check(&root, &paths).unwrap();
    }

    case("cover.jpg", image(10, 10, ImageFormat::Jpeg));
    case("cover.png", image(20, 20, ImageFormat::Png));
  }
}
