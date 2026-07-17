use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Image {
  #[n(0)]
  pub(crate) dimensions: Dimensions,
  #[n(1)]
  pub(crate) filename: ComponentBuf,
  #[n(2)]
  #[serde(rename = "type")]
  pub(crate) ty: ImageType,
}

impl Image {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result<Dimensions> {
    let actual = self.decode(root)?;

    ensure! {
      self.dimensions == actual,
      error::ImageDimensionsMismatch {
        actual,
        expected: self.dimensions,
        path: root.join(self.as_path()),
      },
    }

    Ok(actual)
  }

  fn decode(&self, root: &Utf8Path) -> Result<Dimensions> {
    let path = root.join(self.as_path());

    match self.ty {
      ImageType::Jpeg => Self::decode_jpeg(&path),
      ImageType::Png => Self::decode_png(&path),
    }
  }

  fn decode_jpeg(path: &Utf8Path) -> Result<Dimensions> {
    let bytes = filesystem::read(path)?;

    let mut decoder = JpegDecoder::new(io::Cursor::new(bytes));

    decoder
      .decode_headers()
      .context(error::ImageDecodeJpeg { path })?;

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
      .context(error::ImageDecodePng { path })?;

    let info = reader.info();

    Ok(Dimensions {
      height: info.height.into(),
      width: info.width.into(),
    })
  }

  pub(crate) fn formats(images: &[Image]) -> Vec<ImageType> {
    let mut formats = Vec::new();

    for image in images {
      if !formats.contains(&image.ty) {
        formats.push(image.ty);
      }
    }

    formats
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    self.dimensions = self.decode(root)?;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }
}

impl FromStr for Image {
  type Err = ComponentError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let filename = s.parse::<ComponentBuf>()?;

    let Some(ty) = filename.extension().and_then(ImageType::from_extension) else {
      return Err(ComponentError::Extension {
        extensions: ImageType::EXTENSIONS,
      });
    };

    Ok(Self {
      dimensions: Dimensions::default(),
      filename,
      ty,
    })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, ::image::ImageFormat};

  fn bytes(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
    let mut buffer = io::Cursor::new(Vec::new());
    ::image::DynamicImage::new_rgb8(width, height)
      .write_to(&mut buffer, image_format)
      .unwrap();
    buffer.into_inner()
  }

  #[test]
  fn check_content_rejects_dimensions_mismatch() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.png"), bytes(1, 1, ImageFormat::Png)).unwrap();

    let image = Image {
      dimensions: Dimensions {
        height: 2,
        width: 2,
      },
      filename: "foo.png".parse().unwrap(),
      ty: ImageType::Png,
    };

    assert_matches_regex!(
      image.check_content(&root).unwrap_err().to_string(),
      r"^image `.*foo\.png` is 1×1 but metadata dimensions are 2×2$",
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      "foo.png".parse::<Image>().unwrap(),
      "a300a2000001000167666f6f2e706e670201",
    );

    assert_cbor(
      Image {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      },
      "a300a2000101020167666f6f2e6a70670200",
    );
  }

  #[test]
  fn formats() {
    let foo = "foo.png".parse::<Image>().unwrap();
    let bar = "bar.jpg".parse::<Image>().unwrap();
    let baz = "baz.png".parse::<Image>().unwrap();

    assert_eq!(
      Image::formats(&[foo, bar, baz]),
      [ImageType::Png, ImageType::Jpeg],
    );
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: ComponentError) {
      assert_eq!(s.parse::<Image>().unwrap_err(), expected);
    }

    assert_eq!(
      "foo.jpg".parse::<Image>().unwrap(),
      Image {
        dimensions: Dimensions {
          height: 0,
          width: 0,
        },
        filename: "foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      },
    );

    assert_eq!("foo.png".parse::<Image>().unwrap().ty, ImageType::Png);

    case(
      "foo.svg",
      ComponentError::Extension {
        extensions: &["jpg", "png"],
      },
    );
    case(
      "foo",
      ComponentError::Extension {
        extensions: &["jpg", "png"],
      },
    );
    case("", ComponentError::Empty);
    case("foo/bar.png", ComponentError::Separator { character: '/' });
  }

  #[test]
  fn populate() {
    #[track_caller]
    fn case(filename: &str, bytes: &[u8]) -> Result<Image> {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(filename), bytes).unwrap();

      let mut image = filename.parse::<Image>().unwrap();

      image.populate(&root).map(|()| image)
    }

    assert_eq!(
      case("foo.png", &bytes(2, 1, ImageFormat::Png))
        .unwrap()
        .dimensions,
      Dimensions {
        height: 1,
        width: 2,
      },
    );

    assert_eq!(
      case("foo.jpg", &bytes(1, 2, ImageFormat::Jpeg))
        .unwrap()
        .dimensions,
      Dimensions {
        height: 2,
        width: 1,
      },
    );

    assert_matches_regex!(
      case("foo.png", b"bar").unwrap_err().to_string(),
      r"^failed to decode PNG image `.*foo\.png`$",
    );

    assert_matches_regex!(
      case("foo.jpg", b"bar").unwrap_err().to_string(),
      r"^failed to decode JPEG image `.*foo\.jpg`$",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Image {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      })
      .unwrap(),
      r#"{"dimensions":{"height":1,"width":2},"filename":"foo.jpg","type":"jpeg"}"#,
    );
  }
}
