use super::*;

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Metadata {
  pub(crate) artwork: Option<RelativePath>,
  pub(crate) creator: Option<Text>,
  pub(crate) description: Option<Text>,
  pub(crate) homepage: Option<CheckedUrl>,
  pub(crate) language: Option<Language>,
  pub(crate) media: Option<Media>,
  pub(crate) package: Option<Package>,
  pub(crate) publisher: Option<Text>,
  pub(crate) readme: Option<RelativePath>,
  pub(crate) time: Option<Time>,
  pub(crate) title: Option<Text>,
}

impl Metadata {
  pub(crate) fn deserialize(path: &Utf8Path, yaml: &str) -> Result<Self> {
    serde_yaml::from_str::<Self>(yaml).context(error::DeserializeMetadata { path })
  }

  pub(crate) fn load(self, root: &Utf8Path, quiet: bool) -> Result<crate::Metadata> {
    let Self {
      artwork,
      creator,
      description,
      homepage,
      language,
      media,
      package,
      publisher,
      readme,
      time,
      title,
    } = self;

    let mut files = u64::from(artwork.is_some());

    if let Some(media) = &media {
      match media {
        Media::Audio { items } => files += items.len().into_u64(),
        Media::Image { items } => files += items.len().into_u64(),
        Media::Video { items } => {
          for item in items {
            files += 1 + u64::from(item.placeholder.is_some());
          }
        }
        Media::Web => {}
      }
    }

    let bar = ProgressBar::count(quiet, files, "files");

    let artwork = if let Some(path) = artwork {
      let image = crate::Image::load(root, path)?.content;
      bar.inc(1);
      Some(image)
    } else {
      None
    };

    let media = if let Some(media) = media {
      Some(media.load(root, &bar)?)
    } else {
      None
    };

    Ok(crate::Metadata {
      artwork,
      creator,
      description,
      homepage,
      language,
      media,
      package: package.map(Into::into),
      publisher,
      readme,
      thumbnails: None,
      time,
      title,
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, ::image::ImageFormat};

  #[test]
  fn deserialize_media_audio() {
    let metadata = Metadata::deserialize(
      crate::Metadata::YAML_FILENAME.as_ref(),
      &unindent(
        "
          media:
            type: audio
            items:
              - path: foo.flac
              - path: bar.flac
        ",
      ),
    )
    .unwrap();

    assert_eq!(
      metadata.media,
      Some(Media::Audio {
        items: vec![
          Audio {
            path: "foo.flac".parse().unwrap(),
          },
          Audio {
            path: "bar.flac".parse().unwrap(),
          },
        ],
      }),
    );
  }

  #[test]
  fn deserialize_media_video() {
    let metadata = Metadata::deserialize(
      crate::Metadata::YAML_FILENAME.as_ref(),
      &unindent(
        "
          media:
            type: video
            items:
              - path: foo.mp4
              - path: bar.mp4
                placeholder: baz.png
        ",
      ),
    )
    .unwrap();

    assert_eq!(
      metadata.media,
      Some(Media::Video {
        items: vec![
          Video {
            placeholder: None,
            path: "foo.mp4".parse().unwrap(),
          },
          Video {
            placeholder: Some("baz.png".parse().unwrap()),
            path: "bar.mp4".parse().unwrap(),
          },
        ],
      }),
    );
  }

  #[test]
  fn deserialize_media_web() {
    let metadata = Metadata::deserialize(
      crate::Metadata::YAML_FILENAME.as_ref(),
      &unindent(
        "
          media:
            type: web
        ",
      ),
    )
    .unwrap();

    assert_eq!(metadata.media, Some(Media::Web));
  }

  #[test]
  fn deserialize_rejects_invalid_values() {
    #[track_caller]
    fn case(yaml: &str, expected: &str) {
      let error = Metadata::deserialize(crate::Metadata::YAML_FILENAME.as_ref(), &unindent(yaml))
        .unwrap_err();

      let chain = error
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": ");

      assert_matches_regex!(chain, format!(".*{}.*", expected));
    }

    case(
      "
        title: Foo
        time: 2024/06/15
      ",
      "time: invalid time `2024/06/15`",
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
        homepage: ftp://example.com
      ",
      "homepage: URL scheme `ftp` not allowed, must be `http` or `https`",
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
          time: not-a-time
      ",
      r"package\.time: invalid time `not-a-time`",
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
        description: \"foo\\tbar\"
      ",
      r"description: text may not contain control character `\\t`",
    );
    case(
      "
        title: \"foo\\nbar\"
      ",
      r"title: text may not contain control character `\\n`",
    );
    case(
      "
        title: Foo
        package:
          creator: \"foo\\nbar\"
      ",
      r"package\.creator: text may not contain control character `\\n`",
    );
    case(
      "
        title: Foo
        package:
          description: \"foo\\tbar\"
      ",
      r"package\.description: text may not contain control character `\\t`",
    );
    case(
      "
        media:
          type: video
          items:
            - path: foo.mp4
              bar: baz
      ",
      "unknown field `bar`, expected `path` or `placeholder`",
    );
    case(
      "
        media:
          type: audio
          items:
            - path: foo.flac
              bar: baz
      ",
      "unknown field `bar`, expected `path`",
    );
  }

  #[test]
  fn deserialize_rejects_unknown_fields() {
    #[track_caller]
    fn case(yaml: &str, expected: &str) {
      let chain = Metadata::deserialize(crate::Metadata::YAML_FILENAME.as_ref(), yaml)
        .unwrap_err()
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": ");

      assert_matches_regex!(chain, expected);
    }

    case(
      "title: foo\nbar: 1",
      ".*unknown field `bar`, expected one of .*",
    );
    case(
      "package:\n  bar: 1",
      ".*unknown field `bar`, expected one of .*",
    );
    case(
      "thumbnails:\n  foo.png: thumbnails/foo.jpg",
      ".*unknown field `thumbnails`, expected one of .*",
    );
  }

  #[test]
  fn filepack_metadata_is_valid() {
    Metadata::deserialize(
      crate::Metadata::YAML_FILENAME.as_ref(),
      &filesystem::read_to_string(crate::Metadata::YAML_FILENAME).unwrap(),
    )
    .unwrap();
  }

  fn image(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
    let mut buffer = io::Cursor::new(Vec::new());
    gradient(width, height)
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

      assert_matches_regex!(
        metadata
          .load(&root, true)
          .and_then(|metadata| metadata.validate(&root))
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

      let metadata = Metadata {
        media: Some(Media::Image {
          items: vec![Image {
            path: filename.parse().unwrap(),
          }],
        }),
        ..default()
      };

      assert_matches_regex!(
        metadata.load(&root, true).unwrap_err().to_string(),
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
  fn load() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("cover.png"), image(1, 1, ImageFormat::Png)).unwrap();

    std::fs::write(
      root.join("foo.flac"),
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "1")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "1")
        .tag("TRACKTOTAL", "1")
        .samples(1)
        .build(),
    )
    .unwrap();

    let metadata = Metadata {
      artwork: Some("cover.png".parse().unwrap()),
      media: Some(Media::Audio {
        items: vec![Audio {
          path: "foo.flac".parse().unwrap(),
        }],
      }),
      package: Some(Package {
        colophon: Some("COLOPHON.md".parse().unwrap()),
        creator: None,
        description: None,
        homepage: None,
        time: None,
        title: Some("baz".parse().unwrap()),
      }),
      title: Some("foo".parse().unwrap()),
      ..default()
    }
    .load(&root, true)
    .unwrap();

    assert_eq!(
      metadata,
      crate::Metadata {
        artwork: Some(crate::Image {
          alpha: false,
          bit_depth: 8,
          chroma_subsampling: None,
          color_type: ColorType::Rgb,
          dimensions: Dimensions {
            height: 1,
            width: 1,
          },
          orientation: Orientation::new(),
          path: "cover.png".parse().unwrap(),
          ty: ImageType::Png,
        }),
        media: Some(crate::Media::Audio {
          items: vec![Item {
            content: crate::Audio {
              album: "qux".parse().unwrap(),
              artist: "baz".parse().unwrap(),
              channels: 2,
              disc: 1,
              discs: 1,
              path: "foo.flac".parse().unwrap(),
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 1,
              size: 1024,
              track: 1,
              tracks: 1,
              ty: AudioType::Flac,
            },
            title: Some("bar".parse().unwrap()),
          }],
        }),
        package: Some(crate::Package {
          colophon: Some("COLOPHON.md".parse().unwrap()),
          creator: None,
          description: None,
          homepage: None,
          time: None,
          title: Some("baz".parse().unwrap()),
        }),
        title: Some("foo".parse().unwrap()),
        ..default()
      },
    );
  }

  #[test]
  fn load_rejects_invalid_extensions() {
    #[track_caller]
    fn case(metadata: Metadata, expected: &str) {
      let (_tempdir, root) = tempdir();

      assert_eq!(
        metadata
          .load(&root, true)
          .unwrap_err()
          .iter_chain()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(": "),
        expected,
      );
    }

    case(
      Metadata {
        artwork: Some("cover.svg".parse().unwrap()),
        ..default()
      },
      "invalid path `cover.svg`: path must end in `.jpg` or `.png`",
    );

    case(
      Metadata {
        media: Some(Media::Audio {
          items: vec![Audio {
            path: "foo.wav".parse().unwrap(),
          }],
        }),
        ..default()
      },
      "invalid path `foo.wav`: path must end in `.flac` or `.mp3`",
    );

    case(
      Metadata {
        media: Some(Media::Image {
          items: vec![Image {
            path: "foo.svg".parse().unwrap(),
          }],
        }),
        ..default()
      },
      "invalid path `foo.svg`: path must end in `.jpg` or `.png`",
    );

    case(
      Metadata {
        media: Some(Media::Video {
          items: vec![Video {
            placeholder: None,
            path: "foo.avi".parse().unwrap(),
          }],
        }),
        ..default()
      },
      "invalid path `foo.avi`: path must end in `.mp4` or `.webm`",
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
        description,
        homepage,
        language,
        media,
        package,
        publisher,
        readme,
        time,
        title,
      } = metadata;

      if title
        .as_ref()
        .is_none_or(|title| title.as_str() != "Tobin's Spirit Guide")
      {
        continue;
      }

      assert!(artwork.is_some());
      assert!(creator.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(language.is_some());
      assert!(publisher.is_some());
      assert!(readme.is_some());
      assert!(time.is_some());
      assert!(title.is_some());

      assert!(media.is_none());

      let Package {
        colophon,
        creator,
        description,
        homepage,
        time,
        title,
      } = package.unwrap();

      assert!(colophon.is_some());
      assert!(creator.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(time.is_some());
      assert!(title.is_some());
    }
  }

  #[test]
  fn valid_artwork() {
    #[track_caller]
    fn case(artwork: &str, bytes: Vec<u8>) {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(artwork), bytes).unwrap();

      let metadata = Metadata {
        artwork: Some(artwork.parse().unwrap()),
        package: Some(Package {
          colophon: Some("COLOPHON.md".parse().unwrap()),
          creator: None,
          description: None,
          homepage: None,
          time: None,
          title: None,
        }),
        readme: Some("README.md".parse().unwrap()),
        ..default()
      };

      let paths = [artwork, "README.md", "COLOPHON.md"]
        .into_iter()
        .map(|path| path.parse::<RelativePath>().unwrap())
        .collect();

      let metadata = metadata.load(&root, true).unwrap();
      metadata.check_files(&paths).unwrap();
      metadata.validate(&root).unwrap();
    }

    case("cover.jpg", image(10, 10, ImageFormat::Jpeg));
    case("cover.png", image(20, 20, ImageFormat::Png));
  }

  #[test]
  fn valid_images() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.jpg"), image(2, 1, ImageFormat::Jpeg)).unwrap();
    std::fs::write(root.join("bar.png"), image(1, 2, ImageFormat::Png)).unwrap();

    let metadata = Metadata {
      media: Some(Media::Image {
        items: vec![
          Image {
            path: "foo.jpg".parse().unwrap(),
          },
          Image {
            path: "bar.png".parse().unwrap(),
          },
        ],
      }),
      ..default()
    };

    let paths = ["foo.jpg", "bar.png"]
      .into_iter()
      .map(|path| path.parse::<RelativePath>().unwrap())
      .collect();

    let metadata = metadata.load(&root, true).unwrap();
    metadata.check_files(&paths).unwrap();
    metadata.validate(&root).unwrap();
  }

  #[test]
  fn valid_video_placeholder() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.mp4"),
      Mp4Builder::new().video_track(2, 1).build(),
    )
    .unwrap();
    std::fs::write(root.join("bar.png"), image(2, 1, ImageFormat::Png)).unwrap();

    let metadata = Metadata {
      media: Some(Media::Video {
        items: vec![Video {
          placeholder: Some("bar.png".parse().unwrap()),
          path: "foo.mp4".parse().unwrap(),
        }],
      }),
      ..default()
    };

    let paths = ["foo.mp4", "bar.png"]
      .into_iter()
      .map(|path| path.parse::<RelativePath>().unwrap())
      .collect();

    let metadata = metadata.load(&root, true).unwrap();

    let Some(crate::Media::Video { items }) = &metadata.media else {
      panic!();
    };

    assert_eq!(
      items[0].content.placeholder.as_ref().unwrap().path,
      "bar.png".parse::<RelativePath>().unwrap(),
    );

    metadata.check_files(&paths).unwrap();
    metadata.validate(&root).unwrap();
  }
}
