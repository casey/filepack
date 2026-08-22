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
  pub(crate) orientation: Orientation,
  #[n(6)]
  pub(crate) path: RelativePath,
  #[n(7)]
  #[serde(rename = "type")]
  pub(crate) ty: ImageType,
}

impl Image {
  const THUMBNAIL_QUALITY: u8 = 80;
  const THUMBNAIL_SIZE: u32 = 1024;

  pub(crate) fn create_thumbnail(&self, root: &Utf8Path) -> Result<RelativePath> {
    use ::image::{
      DynamicImage, ImageDecoder, ImageFormat, ImageReader, codecs::jpeg::JpegEncoder,
      imageops::FilterType,
    };

    let destination = self.default_thumbnail_path()?;

    let path = &root.join(&self.path);

    let format = match self.ty {
      ImageType::Jpeg => ImageFormat::Jpeg,
      ImageType::Png => ImageFormat::Png,
    };

    let mut decoder = ImageReader::with_format(io::Cursor::new(filesystem::read(path)?), format)
      .into_decoder()
      .context(error::ThumbnailGeneration { path })?;

    let orientation = decoder
      .orientation()
      .context(error::ThumbnailGeneration { path })?;

    let image = DynamicImage::from_decoder(decoder).context(error::ThumbnailGeneration { path })?;

    let mut thumbnail = image.resize(
      Self::THUMBNAIL_SIZE,
      Self::THUMBNAIL_SIZE,
      FilterType::Lanczos3,
    );

    thumbnail.apply_orientation(orientation);

    let thumbnail = thumbnail.into_rgb8();

    let mut jpeg = Vec::new();

    JpegEncoder::new_with_quality(&mut jpeg, Self::THUMBNAIL_QUALITY)
      .encode_image(&thumbnail)
      .context(error::ThumbnailGeneration { path })?;

    {
      let destination = root.join(&destination);

      filesystem::create_dir_all(destination.parent().unwrap())?;

      filesystem::write(&destination, jpeg)?;
    }

    Ok(destination)
  }

  fn decode(&self, root: &Utf8Path) -> Result<ImageMetadata> {
    let path = root.join(&self.path);

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

  pub(crate) fn default_thumbnail_path(&self) -> Result<RelativePath> {
    let path = format!("thumbnails/{}.jpg", self.path.stem());
    path.parse().context(error::Path { path: &path })
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
  type Err = PathError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let path = s.parse::<RelativePath>()?;

    let Some(ty) = path.extension().and_then(ImageType::from_extension) else {
      return Err(PathError::Extension {
        extensions: ImageType::EXTENSIONS,
      });
    };

    Ok(Self {
      alpha: false,
      bit_depth: 0,
      chroma_subsampling: None,
      color_type: ColorType::default(),
      dimensions: Dimensions::default(),
      orientation: Orientation::new(),
      path,
      ty,
    })
  }
}

impl Item for Image {
  fn info(&self, url: String) -> Info {
    InfoBuilder::new()
      .link("path", &self.path, url)
      .value("type", self.ty)
      .value("dimensions", self.dimensions)
      .value("orientation", self.orientation)
      .value("color type", self.color_type)
      .value("bit depth", format!("{}-bit", self.bit_depth))
      .optional("chroma subsampling", self.chroma_subsampling)
      .value("alpha", self.alpha)
      .build()
  }

  fn path(&self) -> &RelativePath {
    &self.path
  }

  fn resource_type(&self) -> ResourceType {
    self.resource_type()
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    ::image::{DynamicImage, ImageEncoder, ImageFormat, codecs::jpeg::JpegEncoder},
  };

  #[test]
  fn create_thumbnail() {
    #[track_caller]
    fn case(source: (u32, u32), expected: (u32, u32)) {
      let (_tempdir, root) = tempdir();

      DynamicImage::new_rgb8(source.0, source.1)
        .save_with_format(root.join("foo.png"), ImageFormat::Png)
        .unwrap();

      let destination = "foo.png"
        .parse::<Image>()
        .unwrap()
        .create_thumbnail(&root)
        .unwrap();

      assert_eq!(destination, "thumbnails/foo.jpg");

      let thumbnail = ::image::open(root.join(&destination)).unwrap();

      assert_eq!((thumbnail.width(), thumbnail.height()), expected);
    }

    case((1280, 640), (1024, 512));
    case((1, 2), (512, 1024));
  }

  #[test]
  fn create_thumbnail_orientation() {
    let (_tempdir, root) = tempdir();

    let mut encoded = Vec::new();

    let mut encoder = JpegEncoder::new(&mut encoded);

    encoder.set_exif_metadata(exif(6)).unwrap();

    encoder.encode_image(&DynamicImage::new_rgb8(4, 2)).unwrap();

    std::fs::write(root.join("foo.jpg"), encoded).unwrap();

    let destination = "foo.jpg"
      .parse::<Image>()
      .unwrap()
      .create_thumbnail(&root)
      .unwrap();

    let thumbnail = ::image::open(root.join(&destination)).unwrap();

    assert_eq!((thumbnail.width(), thumbnail.height()), (512, 1024));
  }

  #[test]
  fn default_thumbnail_path() {
    assert_eq!(
      "foo/bar baz.png"
        .parse::<Image>()
        .unwrap()
        .default_thumbnail_path()
        .unwrap(),
      "thumbnails/bar baz.jpg",
    );
  }

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
      orientation: Orientation::new(),
      path: "foo.png".parse().unwrap(),
      ty: ImageType::Png,
    };

    let bar = Image {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: None,
      color_type: ColorType::Rgb,
      dimensions: Dimensions::default(),
      orientation: Orientation::new(),
      path: "bar.jpg".parse().unwrap(),
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
      orientation: Orientation::new(),
      path: "baz.png".parse().unwrap(),
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
    fn case(s: &str, expected: PathError) {
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
        orientation: Orientation::new(),
        path: "foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      },
    );

    assert_eq!("foo.png".parse::<Image>().unwrap().ty, ImageType::Png);

    assert_eq!(
      "foo/bar.png".parse::<Image>().unwrap().path,
      "foo/bar.png".parse::<RelativePath>().unwrap(),
    );

    case(
      "foo.svg",
      PathError::Extension {
        extensions: &["jpg", "png"],
      },
    );
    case(
      "foo",
      PathError::Extension {
        extensions: &["jpg", "png"],
      },
    );
    case("", PathError::Empty);
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
      case(
        "foo.png",
        &PngBuilder::new().width(2).height(3).exif(&exif(5)).build(),
      )
      .unwrap()
      .dimensions,
      Dimensions {
        height: 3,
        width: 2,
      },
    );

    assert_eq!(
      case(
        "foo.jpg",
        &JpegBuilder::new().height(2).exif(&exif(5)).build(),
      )
      .unwrap()
      .dimensions,
      Dimensions {
        height: 2,
        width: 1,
      },
    );

    assert_eq!(
      case(
        "foo.jpg",
        &JpegBuilder::new().width(2).exif(&exif(6)).build(),
      )
      .unwrap(),
      Image {
        alpha: false,
        bit_depth: 8,
        chroma_subsampling: Some(ChromaSubsampling::Yuv444),
        color_type: ColorType::Rgb,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        orientation: Orientation {
          mirrored: false,
          rotation: Rotation::R90,
        },
        path: "foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      },
    );

    assert_eq!(
      case(
        "foo.png",
        &PngBuilder::new().width(2).exif(&exif(5)).build(),
      )
      .unwrap(),
      Image {
        alpha: false,
        bit_depth: 8,
        chroma_subsampling: None,
        color_type: ColorType::Rgb,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        orientation: Orientation {
          mirrored: true,
          rotation: Rotation::R90,
        },
        path: "foo.png".parse().unwrap(),
        ty: ImageType::Png,
      },
    );

    let image = case("foo.jpg", &JpegBuilder::new().grayscale().build()).unwrap();
    assert!(!image.alpha);
    assert_eq!(image.bit_depth, 8);
    assert_eq!(image.chroma_subsampling, Some(ChromaSubsampling::Yuv400));
    assert_eq!(image.color_type, ColorType::Grayscale);

    assert_eq!(
      case("foo.jpg", &JpegBuilder::new().sampling(0x22).build())
        .unwrap()
        .chroma_subsampling,
      Some(ChromaSubsampling::Yuv420),
    );

    assert_matches_regex!(
      case("foo.jpg", &JpegBuilder::new().sampling(0x41).build())
        .unwrap_err()
        .to_string(),
      r"^unsupported chroma subsampling 4×1 in image `.*foo\.jpg`$",
    );

    let image = case(
      "foo.png",
      &PngBuilder::new()
        .color(png::ColorType::Rgba)
        .depth(png::BitDepth::Sixteen)
        .build(),
    )
    .unwrap();
    assert!(image.alpha);
    assert_eq!(image.bit_depth, 16);
    assert_eq!(image.chroma_subsampling, None);
    assert_eq!(image.color_type, ColorType::Rgb);

    let image = case(
      "foo.png",
      &PngBuilder::new()
        .color(png::ColorType::Indexed)
        .depth(png::BitDepth::One)
        .build(),
    )
    .unwrap();
    assert!(!image.alpha);
    assert_eq!(image.bit_depth, 1);
    assert_eq!(image.color_type, ColorType::Indexed);

    assert!(
      case(
        "foo.png",
        &PngBuilder::new()
          .color(png::ColorType::Indexed)
          .depth(png::BitDepth::One)
          .trns(&[0])
          .build(),
      )
      .unwrap()
      .alpha
    );

    let image = case(
      "foo.png",
      &PngBuilder::new()
        .color(png::ColorType::GrayscaleAlpha)
        .build(),
    )
    .unwrap();
    assert!(image.alpha);
    assert_eq!(image.color_type, ColorType::Grayscale);

    let image = case(
      "foo.png",
      &PngBuilder::new()
        .color(png::ColorType::Grayscale)
        .depth(png::BitDepth::Two)
        .build(),
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
      case("foo.jpg", &JpegBuilder::new().width(2).exif(b"foo").build())
        .unwrap_err()
        .to_string(),
      r"^invalid EXIF in image `.*foo\.jpg`$",
    );

    assert_matches_regex!(
      case("foo.png", &PngBuilder::new().width(2).exif(b"foo").build())
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
        orientation: Orientation {
          mirrored: true,
          rotation: Rotation::R90,
        },
        path: "foo.jpg".parse().unwrap(),
        ty: ImageType::Jpeg,
      })
      .unwrap(),
      r#"{"alpha":false,"bit_depth":8,"chroma_subsampling":"4:2:0","color_type":"rgb","dimensions":{"height":1,"width":2},"orientation":{"mirrored":true,"rotation":90},"path":"foo.jpg","type":"jpeg"}"#,
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
        orientation: Orientation::new(),
        path: "foo.png".parse().unwrap(),
        ty: ImageType::Png,
      })
      .unwrap(),
      r#"{"alpha":true,"bit_depth":16,"color_type":"rgb","dimensions":{"height":1,"width":2},"orientation":{"mirrored":false,"rotation":0},"path":"foo.png","type":"png"}"#,
    );
  }
}
