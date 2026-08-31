use super::*;

#[allow(private_interfaces)]
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Encode, Decode, PartialEq, Serialize)]
pub struct Metadata {
  #[n(0)]
  pub artwork: Option<Image>,
  #[n(1)]
  pub creator: Option<Text>,
  #[n(2)]
  pub description: Option<Text>,
  #[n(3)]
  pub homepage: Option<CheckedUrl>,
  #[n(4)]
  pub language: Option<Language>,
  #[n(5)]
  pub media: Option<Media>,
  #[n(6)]
  pub package: Option<Package>,
  #[n(7)]
  pub publisher: Option<Text>,
  #[n(8)]
  pub readme: Option<RelativePath>,
  #[n(9)]
  pub thumbnails: Option<BTreeMap<RelativePath, Image>>,
  #[n(10)]
  pub time: Option<Time>,
  #[n(11)]
  pub title: Option<Text>,
}

impl Metadata {
  pub(crate) const CBOR_FILENAME: &'static str = "metadata.filemeta";
  pub(crate) const YAML_FILENAME: &'static str = "metadata.yaml";

  fn check_colophon(colophon: &RelativePath) -> Result {
    ensure! {
      colophon.extension().is_some_and(|extension| extension == "md"),
      error::ColophonExtension {
        colophon,
      },
    }

    Ok(())
  }

  pub(crate) fn check_extras(
    &self,
    files: &HashSet<RelativePath>,
    empty: &[RelativePath],
  ) -> Result {
    let present = files
      .iter()
      .cloned()
      .chain(empty.iter().cloned())
      .collect::<HashSet<RelativePath>>();

    let referenced = self
      .files()
      .into_iter()
      .chain(iter::once(Self::YAML_FILENAME.parse().unwrap()))
      .chain(iter::once(Self::CBOR_FILENAME.parse().unwrap()))
      .collect::<HashSet<RelativePath>>();

    let mut extra = present
      .difference(&referenced)
      .cloned()
      .collect::<Vec<RelativePath>>();

    if let Some(Media::Web) = self.media {
      extra.retain(|path| !path.starts_with("static"));
    }

    extra.sort();

    ensure! {
      extra.is_empty(),
      error::ExtraFiles { paths: extra },
    }

    Ok(())
  }

  pub(crate) fn check_files(&self, paths: &HashSet<RelativePath>) -> Result {
    if let Some(media) = &self.media
      && media.ty().has_items()
    {
      let mut seen = HashSet::new();
      for item in media.items() {
        let path = item.path();
        ensure! {
          seen.insert(path),
          error::DuplicateItem { path },
        }
      }
    }

    for path in self.files() {
      ensure! {
        paths.contains(&path),
        error::MissingMetadataFile { path },
      }
    }

    Ok(())
  }

  fn check_readme(readme: &RelativePath) -> Result {
    ensure! {
      readme.extension().is_some_and(|extension| extension == "md"),
      error::ReadmeExtension {
        readme,
      },
    }

    Ok(())
  }

  pub(crate) fn files(&self) -> Vec<RelativePath> {
    let mut files = Vec::new();

    if let Some(artwork) = &self.artwork {
      files.push(artwork.path.clone());
    }

    if let Some(package) = &self.package
      && let Some(colophon) = &package.colophon
    {
      files.push(colophon.clone());
    }

    if let Some(readme) = &self.readme {
      files.push(readme.clone());
    }

    if let Some(media) = &self.media {
      match media {
        Media::Audio { items } => {
          files.extend(items.iter().map(|audio| audio.path().into()));
        }
        Media::Image { items } => {
          files.extend(items.iter().map(|image| image.path().into()));
        }
        Media::Video { items } => {
          for video in items {
            files.push(video.path().into());
            if let Some(placeholder) = &video.content.placeholder {
              files.push(placeholder.path.clone());
            }
          }
        }
        Media::Web => files.push("static/index.html".parse().unwrap()),
      }
    }

    if let Some(thumbnails) = &self.thumbnails {
      files.extend(thumbnails.values().map(|image| image.path.clone()));
    }

    files
  }

  pub(crate) fn generate(&mut self, root: &Utf8Path, force: bool, quiet: bool) -> Result {
    assert!(self.thumbnails.is_none());

    let mut images = Vec::new();

    if let Some(artwork) = &self.artwork {
      images.push(artwork);
    }

    if let Some(Media::Image { items }) = &self.media {
      images.extend(items.iter().map(|item| &item.content));
    }

    if let Some(Media::Video { items }) = &self.media {
      for item in items {
        if let Some(placeholder) = &item.content.placeholder {
          images.push(placeholder);
        }
      }
    }

    if images.is_empty() {
      return Ok(());
    }

    let mut existing = HashMap::new();

    {
      let path = &root.join(Image::THUMBNAIL_DIR);
      if !force && filesystem::exists(path)? {
        for entry in path.read_dir_utf8().context(error::FilesystemIo { path })? {
          let entry = entry
            .context(error::FilesystemIo { path })?
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_owned();
          existing.insert(entry.file_stem().unwrap().to_owned(), entry);
        }
      }
    }

    let mut destinations = HashMap::new();

    for image in &images {
      if let Some(path) = existing.get(image.path.stem()) {
        return Err(
          error::ThumbnailAlreadyExists {
            image: &image.path,
            path,
          }
          .build(),
        );
      }

      if let Some(first) = destinations.insert(image.thumbnail_stem(), image.path.clone()) {
        return Err(
          error::ThumbnailCollision {
            first,
            second: &image.path,
          }
          .build(),
        );
      }
    }

    let bar = ProgressBar::count(quiet, images.len().into_u64(), "thumbnails");

    let mut thumbnails = BTreeMap::new();

    for image in images {
      let thumbnail = image.create_thumbnail(root)?;

      bar.inc(1);

      let Some(thumbnail) = thumbnail else {
        continue;
      };

      let thumbnail = Image::load(root, thumbnail)?.content;

      thumbnails.insert(image.path.clone(), thumbnail);
    }

    if !thumbnails.is_empty() {
      self.thumbnails = Some(thumbnails);
    }

    Ok(())
  }

  pub(crate) fn info(
    &self,
    builder: InfoBuilder,
    fingerprint: Fingerprint,
    readme: Option<Hash>,
    colophon: Option<Hash>,
  ) -> InfoBuilder {
    builder
      .optional("title", self.title.as_ref())
      .optional("creator", self.creator.as_ref())
      .optional("publisher", self.publisher.as_ref())
      .optional("time", self.time.as_ref())
      .optional("description", self.description.as_ref())
      .when_some(self.readme.as_ref().zip(readme), |builder, (path, hash)| {
        builder.link(
          "readme",
          "view",
          format!("/file/{hash}/{}", path.percent_encode_path()),
        )
      })
      .when_some(self.homepage.as_ref(), |builder, homepage| {
        builder.link("homepage", homepage, homepage.to_string())
      })
      .optional("language", self.language.map(Language::name))
      .when_some(
        self
          .package
          .as_ref()
          .filter(|package| **package != Package::default()),
        |builder, package| builder.info("package", package.info(colophon)),
      )
      .when_some(self.media.as_ref(), |builder, media| {
        media.info(builder, fingerprint)
      })
  }

  pub(crate) fn thumbnail(&self, path: &RelativePath) -> Option<&Image> {
    self.thumbnails.as_ref()?.get(path)
  }

  pub(crate) fn validate(&self, root: &Utf8Path) -> Result {
    if let Some(readme) = &self.readme {
      Self::check_readme(readme)?;
    }

    if let Some(package) = &self.package
      && let Some(colophon) = &package.colophon
    {
      Self::check_colophon(colophon)?;
    }

    if let Some(artwork) = &self.artwork {
      ensure! {
        artwork.dimensions.width == artwork.dimensions.height,
        error::ArtworkAspectRatio {
          dimensions: artwork.dimensions,
          path: root.join(&artwork.path),
        }
      }
    }

    if let Some(Media::Audio { items }) = &self.media {
      Audio::check_positions(items).context(error::AudioPosition)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, ::image::ImageFormat};

  fn colophon_package(colophon: &str) -> Package {
    Package {
      colophon: Some(colophon.parse().unwrap()),
      creator: None,
      description: None,
      homepage: None,
      time: None,
      title: None,
    }
  }

  #[test]
  fn duplicate_items() {
    assert_eq!(
      Metadata {
        media: Some(Media::Image {
          items: vec![Item::test("foo.png"), Item::test("foo.png")],
        }),
        ..default()
      }
      .check_files(&["foo.png".parse().unwrap()].into())
      .unwrap_err()
      .to_string(),
      "duplicate item in metadata: `foo.png`",
    );
  }

  #[test]
  fn encoding() {
    assert_encoding(Metadata {
      artwork: Some(Image {
        alpha: true,
        bit_depth: 8,
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        color_type: ColorType::Rgb,
        dimensions: Dimensions {
          height: 1,
          width: 1,
        },
        orientation: Orientation::new(),
        path: "cover.png".parse().unwrap(),
        ty: ImageType::Png,
      }),
      creator: Some("foo".parse().unwrap()),
      description: Some("bar".parse().unwrap()),
      homepage: Some("http://example.com".parse().unwrap()),
      language: Some("en".parse().unwrap()),
      media: Some(Media::Audio {
        items: vec![Item {
          content: Audio {
            album: "bar".parse().unwrap(),
            artist: "baz".parse().unwrap(),
            channels: 8,
            disc: 3,
            discs: 4,
            path: "track.flac".parse().unwrap(),
            sample_bits: Some(7),
            sample_rate: 1,
            samples: 2,
            size: 9,
            track: 5,
            tracks: 6,
            ty: AudioType::Flac,
          },
          title: Some("foo".parse().unwrap()),
        }],
      }),
      package: Some(Package {
        colophon: Some("COLOPHON.md".parse().unwrap()),
        creator: Some("baz".parse().unwrap()),
        description: Some("qux".parse().unwrap()),
        homepage: Some("http://example.com/foo".parse().unwrap()),
        time: Some("2024-01-01".parse().unwrap()),
        title: Some("foo-bar".parse().unwrap()),
      }),
      publisher: Some("qux".parse().unwrap()),
      readme: Some("README.md".parse().unwrap()),
      thumbnails: Some(
        [(
          "bar.png".parse().unwrap(),
          Image::test("thumbnails/bar.jpg"),
        )]
        .into(),
      ),
      time: Some("2024".parse().unwrap()),
      title: Some("foo".parse().unwrap()),
    });
  }

  #[test]
  fn files_include_audio_tracks() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec![Item::test("foo.flac"), Item::test("bar.flac")],
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
  fn files_include_images() {
    let metadata = Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png"), Item::test("bar.jpg")],
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

  #[test]
  fn files_include_thumbnails() {
    let metadata = Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png")],
      }),
      thumbnails: Some(
        [(
          "foo.png".parse().unwrap(),
          Image::test("thumbnails/foo.jpg"),
        )]
        .into(),
      ),
      ..default()
    };

    assert_eq!(
      metadata.files(),
      vec![
        "foo.png".parse::<RelativePath>().unwrap(),
        "thumbnails/foo.jpg".parse().unwrap(),
      ],
    );
  }

  #[test]
  fn files_include_videos() {
    let mut bar = Item::<Video>::test("bar.mp4");
    bar.content.placeholder = Some(Image::test("baz.png"));

    let metadata = Metadata {
      media: Some(Media::Video {
        items: vec![Item::test("foo.mp4"), bar],
      }),
      ..default()
    };

    assert_eq!(
      metadata.files(),
      vec![
        "foo.mp4".parse::<RelativePath>().unwrap(),
        "bar.mp4".parse().unwrap(),
        "baz.png".parse().unwrap(),
      ],
    );
  }

  #[test]
  fn generate_includes_artwork() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), image(1280, 640, ImageFormat::Png)).unwrap();
    std::fs::write(root.join("bar.png"), image(1280, 640, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      artwork: Some(Image::test("foo.png")),
      media: Some(Media::Image {
        items: vec![Item::test("bar.png")],
      }),
      ..default()
    };

    metadata.generate(&root, false, true).unwrap();

    assert_eq!(
      metadata
        .thumbnails
        .unwrap()
        .into_iter()
        .map(|(path, thumbnail)| (path, thumbnail.path))
        .collect::<Vec<(RelativePath, RelativePath)>>(),
      vec![
        (
          "bar.png".parse().unwrap(),
          "thumbnails/bar.jpg".parse().unwrap(),
        ),
        (
          "foo.png".parse().unwrap(),
          "thumbnails/foo.jpg".parse().unwrap(),
        ),
      ],
    );
  }

  #[test]
  fn generate_includes_video_placeholders() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("bar.png"), image(1280, 640, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      media: Some(Media::Video {
        items: vec![
          Item {
            content: Video {
              placeholder: Some(Image::test("bar.png")),
              ..Video::test("foo.mp4")
            },
            title: None,
          },
          Item::test("baz.mp4"),
        ],
      }),
      ..default()
    };

    metadata.generate(&root, false, true).unwrap();

    assert_eq!(
      metadata
        .thumbnails
        .unwrap()
        .into_iter()
        .map(|(path, thumbnail)| (path, thumbnail.path))
        .collect::<Vec<(RelativePath, RelativePath)>>(),
      vec![(
        "bar.png".parse().unwrap(),
        "thumbnails/bar.jpg".parse().unwrap(),
      )],
    );
  }

  #[test]
  fn generate_omits_thumbnails_when_all_skipped() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), image(1, 1, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      artwork: Some(Image::test("foo.png")),
      ..default()
    };

    metadata.generate(&root, false, true).unwrap();

    assert_eq!(metadata.thumbnails, None);
  }

  #[test]
  fn generate_populates_thumbnails() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), image(1280, 640, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      artwork: Some(Image::test("foo.png")),
      ..default()
    };

    metadata.generate(&root, false, true).unwrap();

    assert_eq!(
      metadata
        .thumbnails
        .unwrap()
        .into_values()
        .collect::<Vec<Image>>(),
      vec![Image {
        alpha: false,
        bit_depth: 8,
        chroma_subsampling: Some(ChromaSubsampling::Yuv444),
        color_type: ColorType::Rgb,
        dimensions: Dimensions {
          height: 512,
          width: 1024,
        },
        orientation: Orientation::new(),
        path: "thumbnails/foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      }],
    );
  }

  #[test]
  fn generate_rejects_existing_thumbnails() {
    let (_tempdir, root) = tempdir();

    std::fs::create_dir(root.join("thumbnails")).unwrap();
    std::fs::write(root.join("thumbnails/foo.jpg"), "bar").unwrap();

    let mut metadata = Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png")],
      }),
      ..default()
    };

    assert_eq!(
      metadata
        .generate(&root, false, true)
        .unwrap_err()
        .to_string(),
      "thumbnail for `foo.png` conflicts with `thumbnails/foo.jpg`"
        .replace('/', std::path::MAIN_SEPARATOR_STR),
    );
  }

  #[test]
  fn generate_rejects_thumbnail_collisions() {
    let (_tempdir, root) = tempdir();

    let mut metadata = Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.jpg"), Item::test("foo.png")],
      }),
      ..default()
    };

    assert_eq!(
      metadata
        .generate(&root, false, true)
        .unwrap_err()
        .to_string(),
      "thumbnail for `foo.png` conflicts with thumbnail for `foo.jpg`",
    );
  }

  #[test]
  fn generate_skips_larger_thumbnails() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), image(1280, 640, ImageFormat::Png)).unwrap();
    std::fs::write(root.join("bar.png"), image(1, 1, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png"), Item::test("bar.png")],
      }),
      ..default()
    };

    metadata.generate(&root, false, true).unwrap();

    assert_eq!(
      metadata
        .thumbnails
        .unwrap()
        .into_iter()
        .map(|(path, thumbnail)| (path, thumbnail.path))
        .collect::<Vec<(RelativePath, RelativePath)>>(),
      vec![(
        "foo.png".parse().unwrap(),
        "thumbnails/foo.jpg".parse().unwrap(),
      )],
    );

    assert!(!root.join("thumbnails/bar.jpg").exists());
  }

  fn image(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
    let mut buffer = io::Cursor::new(Vec::new());
    gradient(width, height)
      .write_to(&mut buffer, image_format)
      .unwrap();
    buffer.into_inner()
  }

  #[test]
  fn info_omits_empty_package() {
    let metadata = Metadata {
      package: Some(default()),
      ..default()
    };

    assert_eq!(
      metadata
        .info(
          InfoBuilder::new(),
          test::FINGERPRINT.parse().unwrap(),
          None,
          None
        )
        .build(),
      Info::Map(Vec::new()),
    );
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
        artwork: Some(Image::test("cover.png")),
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
        package: Some(colophon_package("COLOPHON.md")),
        ..default()
      },
      "COLOPHON.md",
    );
  }

  #[test]
  fn validate_rejects_invalid_readme_extension() {
    let (_tempdir, root) = tempdir();

    assert_eq!(
      Metadata {
        readme: Some("README.txt".parse().unwrap()),
        ..default()
      }
      .validate(&root)
      .unwrap_err()
      .to_string(),
      "readme `README.txt` must end in `.md`",
    );

    assert_eq!(
      Metadata {
        package: Some(colophon_package("COLOPHON.txt")),
        ..default()
      }
      .validate(&root)
      .unwrap_err()
      .to_string(),
      "colophon `COLOPHON.txt` must end in `.md`",
    );
  }
}
