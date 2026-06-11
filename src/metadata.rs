use super::*;

#[allow(private_interfaces)]
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Encode, Decode, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
  #[n(0)]
  pub artwork: Option<Image>,
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

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result {
    if let Some(artwork) = &self.artwork {
      let dimensions = artwork.check_content(root)?;

      ensure! {
        dimensions.width == dimensions.height,
        error::ArtworkDimensions {
          dimensions,
          path: root.join(artwork.as_path()),
        }
      }
    }

    if let Some(Media::Image { images }) = &self.media {
      for image in images {
        image.check_content(root)?;
      }
    }

    Ok(())
  }

  pub(crate) fn check_files(&self, paths: &HashSet<RelativePath>) -> Result {
    for filename in self.files() {
      ensure! {
        paths.contains(&filename),
        error::MissingMetadataFile { filename },
      }
    }

    Ok(())
  }

  pub(crate) fn deserialize(path: &Utf8Path, yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml).context(error::DeserializeMetadata { path })
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
        Media::Audio { tracks } => files.extend(tracks.iter().map(Track::as_path)),
        Media::Image { images } => files.extend(images.iter().map(Image::as_path)),
      }
    }

    files
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    if let Some(artwork) = &mut self.artwork {
      artwork.populate(root)?;
    }

    match &mut self.media {
      Some(Media::Audio { tracks }) => {
        for track in tracks {
          track.populate(root)?;
        }
      }
      Some(Media::Image { images }) => {
        for image in images {
          image.populate(root)?;
        }
      }
      None => {}
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use ::image::ImageFormat;

  #[test]
  fn deserialize_media_audio() {
    let metadata = Metadata::deserialize(
      Metadata::YAML_FILENAME.as_ref(),
      &unindent(
        "
          media:
            type: audio
            tracks:
              - foo.flac
              - bar.flac
        ",
      ),
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
      let error =
        Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), &unindent(yaml)).unwrap_err();

      let chain = error
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": ");

      assert_matches_regex!(chain, expected);
    }

    case(
      "
        title: Foo
        date: 2024/06/15
      ",
      "date: input contains invalid characters",
    );
    case(
      "
        title: Foo
        homepage: not-a-valid-url
      ",
      "homepage: relative URL without a base",
    );
    case(
      "
        title: Foo
        language: ac
      ",
      "unknown language code `ac`",
    );
    case(
      "
        title: Foo
        package:
          creator_tag: foo
      ",
      r"package\.creator_tag: tags must match regex",
    );
    case(
      "
        title: Foo
        package:
          date: not-a-date
      ",
      r"package\.date: input contains invalid characters",
    );
    case(
      "
        title: Foo
        package:
          homepage: :::invalid
      ",
      "package.homepage: relative URL without a base",
    );
    case(
      "
        title: Foo
        artwork: cover.svg
      ",
      "artwork: component must end in `.jpg` or `.png`",
    );
    case(
      "
        title: Foo
        package:
          nfo: info.txt
      ",
      "nfo: component must end in `.nfo`",
    );
    case(
      "
        title: Foo
        readme: README.txt
      ",
      "readme: component must end in `.md`",
    );
    case(
      "
        title: Foo
        media:
          type: audio
          tracks:
          - foo.mp3
      ",
      r"component must end in `\.flac`",
    );
  }

  #[test]
  fn deserialize_rejects_unknown_fields() {
    #[track_caller]
    fn case(yaml: &str, expected: &str) {
      let chain = Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), yaml)
        .unwrap_err()
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": ");

      assert_matches_regex!(chain, expected);
    }

    case(
      "title: foo\nbar: 1",
      "unknown field `bar`, expected one of ",
    );
    case(
      "package:\n  bar: 1",
      "unknown field `bar`, expected one of ",
    );
  }

  #[test]
  fn encoding() {
    assert_encoding(Metadata {
      artwork: Some(Image {
        dimensions: Dimensions {
          height: 1,
          width: 1,
        },
        filename: "cover.png".parse().unwrap(),
        ty: ImageType::Png,
      }),
      creator: Some("foo".parse().unwrap()),
      date: Some("2024".parse().unwrap()),
      description: Some("bar".into()),
      homepage: Some("http://example.com".parse().unwrap()),
      language: Some("en".parse().unwrap()),
      media: Some(Media::Audio {
        tracks: vec![Track {
          filename: "track.flac".parse().unwrap(),
          title: Some("foo".into()),
          ty: AudioType::Flac,
        }],
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
    Metadata::deserialize(
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

  #[test]
  fn files_includes_images() {
    let metadata = Metadata {
      media: Some(Media::Image {
        images: vec!["foo.png".parse().unwrap(), "bar.jpg".parse().unwrap()],
      }),
      ..default()
    };

    assert_eq!(
      metadata.files(),
      vec![
        "foo.png".parse::<RelativePath>().unwrap(),
        "bar.jpg".parse().unwrap(),
      ],
    );
  }

  fn image(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
    let mut buffer = io::Cursor::new(Vec::new());
    ::image::DynamicImage::new_rgb8(width, height)
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

      let mut metadata = Metadata {
        artwork: Some(filename.parse().unwrap()),
        ..default()
      };

      assert_matches_regex!(
        metadata
          .populate(&root)
          .and_then(|()| metadata.check_content(&root))
          .unwrap_err()
          .to_string(),
        expected
      );
    }

    case(
      "cover.jpg",
      b"bar".to_vec(),
      "failed to decode JPEG image `.*cover\\.jpg`",
    );
    case(
      "cover.png",
      b"bar".to_vec(),
      "failed to decode PNG image `.*cover\\.png`",
    );
    case(
      "cover.jpg",
      image(1, 1, ImageFormat::Png),
      "failed to decode JPEG image `.*cover\\.jpg`",
    );
    case(
      "cover.png",
      image(1, 1, ImageFormat::Jpeg),
      "failed to decode PNG image `.*cover\\.png`",
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
  fn invalid_image() {
    #[track_caller]
    fn case(filename: &str, bytes: Vec<u8>, expected: &str) {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(filename), bytes).unwrap();

      let mut metadata = Metadata {
        media: Some(Media::Image {
          images: vec![filename.parse().unwrap()],
        }),
        ..default()
      };

      assert_matches_regex!(
        metadata
          .populate(&root)
          .and_then(|()| metadata.check_content(&root))
          .unwrap_err()
          .to_string(),
        expected
      );
    }

    case(
      "foo.jpg",
      b"bar".to_vec(),
      "failed to decode JPEG image `.*foo\\.jpg`",
    );
    case(
      "foo.png",
      b"bar".to_vec(),
      "failed to decode PNG image `.*foo\\.png`",
    );
  }

  #[test]
  fn metadata_in_readme_is_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```yaml(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let metadata = Metadata::deserialize("README.md".as_ref(), &capture[1]).unwrap();

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
          .check_files(&HashSet::new())
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
  fn valid_artwork() {
    #[track_caller]
    fn case(artwork: &str, bytes: Vec<u8>) {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(artwork), bytes).unwrap();

      let mut metadata = Metadata {
        artwork: Some(artwork.parse().unwrap()),
        package: Some(nfo_package("info.nfo")),
        readme: Some("README.md".parse().unwrap()),
        ..default()
      };

      let paths = [artwork, "info.nfo", "README.md"]
        .into_iter()
        .map(|path| path.parse::<RelativePath>().unwrap())
        .collect();

      metadata.populate(&root).unwrap();
      metadata.check_files(&paths).unwrap();
      metadata.check_content(&root).unwrap();
    }

    case("cover.jpg", image(10, 10, ImageFormat::Jpeg));
    case("cover.png", image(20, 20, ImageFormat::Png));
  }

  #[test]
  fn valid_images() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.jpg"), image(2, 1, ImageFormat::Jpeg)).unwrap();
    std::fs::write(root.join("bar.png"), image(1, 2, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      media: Some(Media::Image {
        images: vec!["foo.jpg".parse().unwrap(), "bar.png".parse().unwrap()],
      }),
      ..default()
    };

    let paths = ["foo.jpg", "bar.png"]
      .into_iter()
      .map(|path| path.parse::<RelativePath>().unwrap())
      .collect();

    metadata.populate(&root).unwrap();
    metadata.check_files(&paths).unwrap();
    metadata.check_content(&root).unwrap();
  }
}
