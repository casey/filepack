use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Image {
  #[n(0)]
  pub(crate) alpha: bool,
  #[n(1)]
  pub(crate) bit_depth: u64,
  #[n(2)]
  pub(crate) chroma_subsampling: Option<ChromaSubsampling>,
  #[n(3)]
  pub(crate) color_type: ColorType,
  #[n(4)]
  pub(crate) dimensions: Dimensions,
  #[n(5)]
  pub(crate) filename: ComponentBuf,
  #[n(6)]
  pub(crate) orientation: Orientation,
  #[n(7)]
  #[serde(rename = "type")]
  pub(crate) ty: ImageType,
}

impl Image {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  fn decode(&self, root: &Utf8Path) -> Result<ImageMetadata> {
    let path = root.join(self.as_path());

    match self.ty {
      ImageType::Jpeg => Self::decode_jpeg(&path),
      ImageType::Png => Self::decode_png(&path),
    }
  }

  fn decode_jpeg(path: &Utf8Path) -> Result<ImageMetadata> {
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

    let colorspace = decoder.input_colorspace().unwrap();

    let color_type = match colorspace {
      ColorSpace::CMYK | ColorSpace::YCCK => ColorType::Cmyk,
      ColorSpace::Luma => ColorType::Grayscale,
      ColorSpace::YCbCr => ColorType::Rgb,
      colorspace => return Err(error::ImageColorspace { colorspace, path }.build()),
    };

    let chroma_subsampling = if color_type == ColorType::Grayscale {
      ChromaSubsampling::Yuv400
    } else {
      match info.sample_ratio {
        SampleRatios::H => ChromaSubsampling::Yuv422,
        SampleRatios::HV => ChromaSubsampling::Yuv420,
        SampleRatios::None => ChromaSubsampling::Yuv444,
        SampleRatios::V => ChromaSubsampling::Yuv440,
        SampleRatios::Generic(horizontal, vertical) => {
          return Err(
            error::ImageSampleRatio {
              horizontal,
              path,
              vertical,
            }
            .build(),
          );
        }
      }
    };

    Ok(ImageMetadata {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: Some(chroma_subsampling),
      color_type,
      dimensions: Dimensions {
        height: info.height.into(),
        width: info.width.into(),
      },
      orientation,
    })
  }

  fn decode_png(path: &Utf8Path) -> Result<ImageMetadata> {
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

    let (color_type, alpha) = match info.color_type {
      png::ColorType::Grayscale => (ColorType::Grayscale, false),
      png::ColorType::GrayscaleAlpha => (ColorType::Grayscale, true),
      png::ColorType::Indexed => (ColorType::Indexed, false),
      png::ColorType::Rgb => (ColorType::Rgb, false),
      png::ColorType::Rgba => (ColorType::Rgb, true),
    };

    Ok(ImageMetadata {
      alpha: alpha || info.trns.is_some(),
      bit_depth: (info.bit_depth as u8).into(),
      chroma_subsampling: None,
      color_type,
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
    let ImageMetadata {
      alpha,
      bit_depth,
      chroma_subsampling,
      color_type,
      dimensions,
      orientation,
    } = self.decode(root)?;

    self.alpha = alpha;
    self.bit_depth = bit_depth;
    self.chroma_subsampling = chroma_subsampling;
    self.color_type = color_type;
    self.dimensions = dimensions;
    self.orientation = orientation;

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
      alpha: false,
      bit_depth: 0,
      chroma_subsampling: None,
      color_type: ColorType::default(),
      dimensions: Dimensions::default(),
      filename,
      orientation: Orientation::new(),
      ty,
    })
  }
}

impl Item for Image {
  fn info(&self, url: String) -> Info {
    InfoBuilder::new()
      .link("filename", &self.filename, url)
      .value("type", self.ty)
      .value("dimensions", self.dimensions)
      .value("orientation", self.orientation)
      .value("color type", self.color_type)
      .value("bit depth", format!("{}-bit", self.bit_depth))
      .optional("chroma subsampling", self.chroma_subsampling)
      .value("alpha", self.alpha)
      .build()
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
  fn formats() {
    let foo = Image {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: None,
      color_type: ColorType::Rgb,
      dimensions: Dimensions {
        height: 1,
        width: 2,
      },
      filename: "foo.png".parse().unwrap(),
      orientation: Orientation::new(),
      ty: ImageType::Png,
    };

    let bar = Image {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: None,
      color_type: ColorType::Rgb,
      dimensions: Dimensions::default(),
      filename: "bar.jpg".parse().unwrap(),
      orientation: Orientation::new(),
      ty: ImageType::Jpeg,
    };

    let baz = Image {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: None,
      color_type: ColorType::Rgb,
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
        alpha: false,
        bit_depth: 0,
        chroma_subsampling: None,
        color_type: ColorType::Rgb,
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
        alpha: false,
        bit_depth: 8,
        chroma_subsampling: Some(ChromaSubsampling::Yuv444),
        color_type: ColorType::Rgb,
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
        alpha: false,
        bit_depth: 8,
        chroma_subsampling: None,
        color_type: ColorType::Rgb,
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

    let image = case("foo.jpg", &jpeg_grayscale(1, 1)).unwrap();
    assert!(!image.alpha);
    assert_eq!(image.bit_depth, 8);
    assert_eq!(image.chroma_subsampling, Some(ChromaSubsampling::Yuv400));
    assert_eq!(image.color_type, ColorType::Grayscale);

    assert_eq!(
      case("foo.jpg", &jpeg_with_sampling(1, 1, 0x22))
        .unwrap()
        .chroma_subsampling,
      Some(ChromaSubsampling::Yuv420),
    );

    assert_matches_regex!(
      case("foo.jpg", &jpeg_with_sampling(1, 1, 0x41))
        .unwrap_err()
        .to_string(),
      r"^unsupported chroma subsampling 4×1 in image `.*foo\.jpg`$",
    );

    let image = case(
      "foo.png",
      &png(
        1,
        1,
        png::ColorType::Rgba,
        png::BitDepth::Sixteen,
        None,
        None,
      ),
    )
    .unwrap();
    assert!(image.alpha);
    assert_eq!(image.bit_depth, 16);
    assert_eq!(image.chroma_subsampling, None);
    assert_eq!(image.color_type, ColorType::Rgb);

    let image = case(
      "foo.png",
      &png(
        1,
        1,
        png::ColorType::Indexed,
        png::BitDepth::One,
        None,
        None,
      ),
    )
    .unwrap();
    assert!(!image.alpha);
    assert_eq!(image.bit_depth, 1);
    assert_eq!(image.color_type, ColorType::Indexed);

    assert!(
      case(
        "foo.png",
        &png(
          1,
          1,
          png::ColorType::Indexed,
          png::BitDepth::One,
          Some(&[0]),
          None,
        ),
      )
      .unwrap()
      .alpha
    );

    let image = case(
      "foo.png",
      &png(
        1,
        1,
        png::ColorType::GrayscaleAlpha,
        png::BitDepth::Eight,
        None,
        None,
      ),
    )
    .unwrap();
    assert!(image.alpha);
    assert_eq!(image.color_type, ColorType::Grayscale);

    let image = case(
      "foo.png",
      &png(
        1,
        1,
        png::ColorType::Grayscale,
        png::BitDepth::Two,
        None,
        None,
      ),
    )
    .unwrap();
    assert!(!image.alpha);
    assert_eq!(image.bit_depth, 2);
    assert_eq!(image.color_type, ColorType::Grayscale);

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
        alpha: false,
        bit_depth: 8,
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        color_type: ColorType::Rgb,
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
      r#"{"alpha":false,"bit_depth":8,"chroma_subsampling":"4:2:0","color_type":"rgb","dimensions":{"height":1,"width":2},"filename":"foo.jpg","orientation":{"mirrored":true,"rotation":90},"type":"jpeg"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Image {
        alpha: true,
        bit_depth: 16,
        chroma_subsampling: None,
        color_type: ColorType::Rgb,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.png".parse().unwrap(),
        orientation: Orientation::new(),
        ty: ImageType::Png,
      })
      .unwrap(),
      r#"{"alpha":true,"bit_depth":16,"color_type":"rgb","dimensions":{"height":1,"width":2},"filename":"foo.png","orientation":{"mirrored":false,"rotation":0},"type":"png"}"#,
    );
  }
}
