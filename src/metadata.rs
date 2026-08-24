use super::*;

#[allow(private_interfaces)]
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Encode, Decode, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
  pub readme: Option<RelativePath>,
  #[n(8)]
  pub thumbnails: Option<BTreeMap<RelativePath, Image>>,
  #[n(9)]
  pub time: Option<Time>,
  #[n(10)]
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

  pub(crate) fn deserialize(path: &Utf8Path, yaml: &str) -> Result<Self> {
    let metadata =
      serde_yaml::from_str::<Self>(yaml).context(error::DeserializeMetadata { path })?;

    ensure! {
      metadata.thumbnails.is_none(),
      error::DeserializeMetadataThumbnails { path },
    }

    Ok(metadata)
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
        Media::Audio { items } => files.extend(items.iter().map(|audio| audio.path.clone())),
        Media::Image { items } => files.extend(items.iter().map(|image| image.path.clone())),
        Media::Video { items } => files.extend(items.iter().map(|video| video.path.clone())),
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
      images.extend(items);
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

    let bar = progress_bar::count(quiet, images.len().into_u64(), "thumbnails");

    let mut thumbnails = BTreeMap::new();

    for image in images {
      let thumbnail = image.create_thumbnail(root)?;

      bar.inc(1);

      let Some(thumbnail) = thumbnail else {
        continue;
      };

      let thumbnail =
        Image::from_str(thumbnail.as_ref()).context(error::Path { path: thumbnail })?;

      thumbnails.insert(image.path.clone(), thumbnail);
    }

    if !thumbnails.is_empty() {
      self.thumbnails = Some(thumbnails);
    }

    Ok(())
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path, quiet: bool) -> Result {
    let mut files = 0;

    if let Some(media) = &self.media {
      match media {
        Media::Audio { items } => files += items.len().into_u64(),
        Media::Image { items } => files += items.len().into_u64(),
        Media::Video { items } => files += items.len().into_u64(),
        Media::Web => {}
      }
    }

    if self.artwork.is_some() {
      files += 1;
    }

    if let Some(thumbnails) = &self.thumbnails {
      files += thumbnails.len().into_u64();
    }

    let bar = progress_bar::count(quiet, files, "files");

    if let Some(artwork) = &mut self.artwork {
      artwork.populate(root)?;
      bar.inc(1);
    }

    if let Some(media) = self.media.as_mut() {
      match media {
        Media::Audio { items } => {
          for audio in items {
            audio.populate(root)?;
            bar.inc(1);
          }
        }
        Media::Image { items } => {
          for image in items {
            image.populate(root)?;
            bar.inc(1);
          }
        }
        Media::Video { items } => {
          for video in items {
            video.populate(root)?;
            bar.inc(1);
          }
        }
        Media::Web => {}
      }
    }

    if let Some(thumbnails) = &mut self.thumbnails {
      for thumbnail in thumbnails.values_mut() {
        thumbnail.populate(root)?;
        bar.inc(1);
      }
    }

    Ok(())
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
  fn deserialize_media_audio() {
    let metadata = Metadata::deserialize(
      Metadata::YAML_FILENAME.as_ref(),
      &unindent(
        "
          media:
            type: audio
            items:
              - foo.flac
              - bar.flac
        ",
      ),
    )
    .unwrap();

    assert_eq!(
      metadata.media,
      Some(Media::Audio {
        items: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
      }),
    );
  }

  #[test]
  fn deserialize_media_video() {
    let metadata = Metadata::deserialize(
      Metadata::YAML_FILENAME.as_ref(),
      &unindent(
        "
          media:
            type: video
            items:
              - foo.mp4
              - bar.mp4
        ",
      ),
    )
    .unwrap();

    assert_eq!(
      metadata.media,
      Some(Media::Video {
        items: vec!["foo.mp4".parse().unwrap(), "bar.mp4".parse().unwrap()],
      }),
    );
  }

  #[test]
  fn deserialize_media_web() {
    let metadata = Metadata::deserialize(
      Metadata::YAML_FILENAME.as_ref(),
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
      let error =
        Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), &unindent(yaml)).unwrap_err();

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
        artwork: cover.svg
      ",
      "artwork: path must end in `.jpg` or `.png`",
    );
    case(
      "
        title: Foo
        media:
          type: audio
          items:
          - foo.wav
      ",
      r"path must end in `\.flac` or `\.mp3`",
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
  }

  #[test]
  fn deserialize_rejects_thumbnails() {
    assert_eq!(
      Metadata::deserialize(
        Metadata::YAML_FILENAME.as_ref(),
        &unindent(
          "
            thumbnails:
              foo.png: thumbnails/foo.jpg
          ",
        ),
      )
      .unwrap_err()
      .to_string(),
      "metadata at `metadata.yaml` includes thumbnails: thumbnails must be generated",
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
      ".*unknown field `bar`, expected one of .*",
    );
    case(
      "package:\n  bar: 1",
      ".*unknown field `bar`, expected one of .*",
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
        items: vec![Audio {
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
          title: "foo".parse().unwrap(),
          track: 5,
          tracks: 6,
          ty: AudioType::Flac,
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
      readme: Some("README.md".parse().unwrap()),
      thumbnails: Some(
        [(
          "bar.png".parse().unwrap(),
          "thumbnails/bar.jpg".parse().unwrap(),
        )]
        .into(),
      ),
      time: Some("2024".parse().unwrap()),
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
  fn files_include_audio_tracks() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
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
        items: vec!["foo.png".parse().unwrap(), "bar.jpg".parse().unwrap()],
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
        items: vec!["foo.png".parse().unwrap()],
      }),
      thumbnails: Some(
        [(
          "foo.png".parse().unwrap(),
          "thumbnails/foo.jpg".parse().unwrap(),
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
    let metadata = Metadata {
      media: Some(Media::Video {
        items: vec!["foo.mp4".parse().unwrap(), "bar.mp4".parse().unwrap()],
      }),
      ..default()
    };

    assert_eq!(
      metadata.files(),
      vec![
        "foo.mp4".parse::<RelativePath>().unwrap(),
        "bar.mp4".parse().unwrap(),
      ],
    );
  }

  #[test]
  fn generate_includes_artwork() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), image(1280, 640, ImageFormat::Png)).unwrap();
    std::fs::write(root.join("bar.png"), image(1280, 640, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      artwork: Some("foo.png".parse().unwrap()),
      media: Some(Media::Image {
        items: vec!["bar.png".parse().unwrap()],
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
  fn generate_omits_thumbnails_when_all_skipped() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), image(1, 1, ImageFormat::Png)).unwrap();

    let mut metadata = Metadata {
      artwork: Some("foo.png".parse().unwrap()),
      ..default()
    };

    metadata.generate(&root, false, true).unwrap();

    assert_eq!(metadata.thumbnails, None);
  }

  #[test]
  fn generate_rejects_existing_thumbnails() {
    let (_tempdir, root) = tempdir();

    std::fs::create_dir(root.join("thumbnails")).unwrap();
    std::fs::write(root.join("thumbnails/foo.jpg"), "bar").unwrap();

    let mut metadata = Metadata {
      media: Some(Media::Image {
        items: vec!["foo.png".parse().unwrap()],
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
        items: vec!["foo.jpg".parse().unwrap(), "foo.png".parse().unwrap()],
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
        items: vec!["foo.png".parse().unwrap(), "bar.png".parse().unwrap()],
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
          .populate(&root, true)
          .and_then(|()| metadata.validate(&root))
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
          items: vec![filename.parse().unwrap()],
        }),
        ..default()
      };

      assert_matches_regex!(
        metadata.populate(&root, true).unwrap_err().to_string(),
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
        description,
        homepage,
        language,
        media,
        package,
        readme,
        thumbnails,
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
      assert!(readme.is_some());
      assert!(time.is_some());
      assert!(title.is_some());

      assert!(media.is_none());
      assert!(thumbnails.is_none());

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
        package: Some(colophon_package("COLOPHON.md")),
        ..default()
      },
      "COLOPHON.md",
    );
  }

  #[test]
  fn valid_artwork() {
    #[track_caller]
    fn case(artwork: &str, bytes: Vec<u8>) {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(artwork), bytes).unwrap();

      let mut metadata = Metadata {
        artwork: Some(artwork.parse().unwrap()),
        package: Some(colophon_package("COLOPHON.md")),
        readme: Some("README.md".parse().unwrap()),
        ..default()
      };

      let paths = [artwork, "README.md", "COLOPHON.md"]
        .into_iter()
        .map(|path| path.parse::<RelativePath>().unwrap())
        .collect();

      metadata.populate(&root, true).unwrap();
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

    let mut metadata = Metadata {
      media: Some(Media::Image {
        items: vec!["foo.jpg".parse().unwrap(), "bar.png".parse().unwrap()],
      }),
      ..default()
    };

    let paths = ["foo.jpg", "bar.png"]
      .into_iter()
      .map(|path| path.parse::<RelativePath>().unwrap())
      .collect();

    metadata.populate(&root, true).unwrap();
    metadata.check_files(&paths).unwrap();
    metadata.validate(&root).unwrap();
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
