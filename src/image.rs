use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Image {
  #[n(0)]
  pub(crate) dimensions: Dimensions,
  #[n(1)]
  pub(crate) filename: ComponentBuf,
  #[n(2)]
  pub(crate) orientation: Orientation,
  #[n(3)]
  #[serde(rename = "type")]
  pub(crate) ty: ImageType,
}

impl Image {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  fn decode(&self, root: &Utf8Path) -> Result<ImageInfo> {
    let path = root.join(self.as_path());

    match self.ty {
      ImageType::Jpeg => Self::decode_jpeg(&path),
      ImageType::Png => Self::decode_png(&path),
    }
  }

  fn decode_jpeg(path: &Utf8Path) -> Result<ImageInfo> {
    let bytes = filesystem::read(path)?;

    let mut decoder = JpegDecoder::new(io::Cursor::new(bytes));

    decoder
      .decode_headers()
      .context(error::ImageDecodeJpeg { path })?;

    let orientation = if let Some(exif) = decoder.exif() {
      Orientation::from_exif(exif).context(error::ImageExif { path })?
    } else {
      Orientation::new()
    };

    let info = decoder.info().unwrap();

    Ok(ImageInfo {
      dimensions: Dimensions {
        height: info.height.into(),
        width: info.width.into(),
      },
      orientation,
    })
  }

  fn decode_png(path: &Utf8Path) -> Result<ImageInfo> {
    let bytes = filesystem::read(path)?;

    let reader = png::Decoder::new(io::Cursor::new(bytes))
      .read_info()
      .context(error::ImageDecodePng { path })?;

    let info = reader.info();

    let orientation = if let Some(exif) = &info.exif_metadata {
      Orientation::from_exif(exif).context(error::ImageExif { path })?
    } else {
      Orientation::new()
    };

    Ok(ImageInfo {
      dimensions: Dimensions {
        height: info.height.into(),
        width: info.width.into(),
      },
      orientation,
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

  pub(crate) fn oriented_dimensions(&self) -> Dimensions {
    self.orientation.dimensions(self.dimensions)
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let info = self.decode(root)?;

    self.dimensions = info.dimensions;
    self.orientation = info.orientation;

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
      orientation: Orientation::new(),
      ty,
    })
  }
}

impl Item for Image {
  fn info(&self, url: String) -> Info {
    Info::Map(vec![
      (
        "filename".into(),
        Info::Link {
          text: self.filename.to_string(),
          url,
        },
      ),
      ("type".into(), Info::Value(self.ty.to_string())),
      (
        "dimensions".into(),
        Info::Value(self.dimensions.to_string()),
      ),
      (
        "orientation".into(),
        Info::Value(self.orientation.to_string()),
      ),
    ])
  }

  fn path(&self) -> RelativePath {
    self.as_path()
  }

  fn resource_type(&self) -> ResourceType {
    self.resource_type()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      "foo.png".parse::<Image>().unwrap(),
      "a400a2000001000167666f6f2e706e6702a200f401000301",
    );

    assert_cbor(
      Image {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.jpg".parse().unwrap(),
        orientation: Orientation {
          mirrored: true,
          rotation: Rotation::R90,
        },
        ty: ImageType::Jpeg,
      },
      "a400a2000101020167666f6f2e6a706702a200f501010300",
    );
  }

  #[test]
  fn formats() {
    let foo = Image {
      dimensions: Dimensions {
        height: 1,
        width: 2,
      },
      filename: "foo.png".parse().unwrap(),
      orientation: Orientation::new(),
      ty: ImageType::Png,
    };

    let bar = Image {
      dimensions: Dimensions::default(),
      filename: "bar.jpg".parse().unwrap(),
      orientation: Orientation::new(),
      ty: ImageType::Jpeg,
    };

    let baz = Image {
      dimensions: Dimensions {
        height: 3,
        width: 4,
      },
      filename: "baz.png".parse().unwrap(),
      orientation: Orientation::new(),
      ty: ImageType::Png,
    };

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
        orientation: Orientation::new(),
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
  fn oriented_dimensions() {
    let mut image = "foo.png".parse::<Image>().unwrap();

    image.dimensions = Dimensions {
      height: 1,
      width: 2,
    };

    assert_eq!(
      image.oriented_dimensions(),
      Dimensions {
        height: 1,
        width: 2,
      },
    );

    image.orientation.rotation = Rotation::R90;

    assert_eq!(
      image.oriented_dimensions(),
      Dimensions {
        height: 2,
        width: 1,
      },
    );
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
      case("foo.png", &png_with_exif(2, 1, &exif(5)))
        .unwrap()
        .dimensions,
      Dimensions {
        height: 1,
        width: 2,
      },
    );

    assert_eq!(
      case("foo.jpg", &jpeg_with_exif(1, 2, &exif(5)))
        .unwrap()
        .dimensions,
      Dimensions {
        height: 2,
        width: 1,
      },
    );

    assert_eq!(
      case("foo.jpg", &jpeg_with_exif(2, 1, &exif(6))).unwrap(),
      Image {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.jpg".parse().unwrap(),
        orientation: Orientation {
          mirrored: false,
          rotation: Rotation::R90,
        },
        ty: ImageType::Jpeg,
      },
    );

    assert_eq!(
      case("foo.png", &png_with_exif(2, 1, &exif(5))).unwrap(),
      Image {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.png".parse().unwrap(),
        orientation: Orientation {
          mirrored: true,
          rotation: Rotation::R90,
        },
        ty: ImageType::Png,
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

    assert_matches_regex!(
      case("foo.jpg", &jpeg_with_exif(2, 1, b"foo"))
        .unwrap_err()
        .to_string(),
      r"^invalid EXIF in image `.*foo\.jpg`$",
    );

    assert_matches_regex!(
      case("foo.png", &png_with_exif(2, 1, b"foo"))
        .unwrap_err()
        .to_string(),
      r"^invalid EXIF in image `.*foo\.png`$",
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
        orientation: Orientation {
          mirrored: true,
          rotation: Rotation::R90,
        },
        ty: ImageType::Jpeg,
      })
      .unwrap(),
      r#"{"dimensions":{"height":1,"width":2},"filename":"foo.jpg","orientation":{"mirrored":true,"rotation":90},"type":"jpeg"}"#,
    );
  }
}
