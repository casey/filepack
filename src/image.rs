use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
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
  pub(crate) const THUMBNAIL_DIR: &str = "thumbnails";
  const THUMBNAIL_QUALITY: u8 = 80;
  const THUMBNAIL_SIZE: u32 = 1024;

  pub(crate) fn create_thumbnail(&self, root: &Utf8Path) -> Result<Option<RelativePath>> {
    use ::image::{
      DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader,
      codecs::{
        jpeg::JpegEncoder,
        png::{self, CompressionType, PngEncoder},
      },
      imageops,
    };

    let path = &root.join(&self.path);

    let format = match self.ty {
      ImageType::Jpeg => ImageFormat::Jpeg,
      ImageType::Png => ImageFormat::Png,
    };

    let original = filesystem::read(path)?;

    let mut decoder = ImageReader::with_format(io::Cursor::new(&original), format)
      .into_decoder()
      .context(error::ThumbnailGeneration { path })?;

    let orientation = decoder
      .orientation()
      .context(error::ThumbnailGeneration { path })?;

    let mut image =
      DynamicImage::from_decoder(decoder).context(error::ThumbnailGeneration { path })?;

    image.apply_orientation(orientation);

    let thumbnail = if image.width() > Self::THUMBNAIL_SIZE || image.height() > Self::THUMBNAIL_SIZE
    {
      image.resize(
        Self::THUMBNAIL_SIZE,
        Self::THUMBNAIL_SIZE,
        imageops::FilterType::Lanczos3,
      )
    } else {
      image
    };

    let uses_alpha =
      thumbnail.color().has_alpha() && thumbnail.pixels().any(|(_x, _y, pixel)| pixel[3] < u8::MAX);

    let mut encoded = Vec::new();

    let ty = if uses_alpha {
      thumbnail
        .into_rgba8()
        .write_with_encoder(PngEncoder::new_with_quality(
          &mut encoded,
          CompressionType::Best,
          png::FilterType::Adaptive,
        ))
        .context(error::ThumbnailGeneration { path })?;

      ImageType::Png
    } else {
      JpegEncoder::new_with_quality(&mut encoded, Self::THUMBNAIL_QUALITY)
        .encode_image(&thumbnail.into_rgb8())
        .context(error::ThumbnailGeneration { path })?;

      ImageType::Jpeg
    };

    if encoded.len() >= original.len() {
      return Ok(None);
    }

    let destination = self.thumbnail_path(ty)?;

    {
      let destination = root.join(&destination);

      filesystem::create_dir_all(destination.parent().unwrap())?;

      filesystem::write(&destination, encoded)?;
    }

    Ok(Some(destination))
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

    let title = decoder
      .xmp()
      .map(|xmp| {
        let title = xmp::title(xmp).context(error::ImageXmp { path })?;
        Self::title(title, path)
      })
      .transpose()?
      .flatten();

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
      title,
    })
  }

  fn decode_png(path: &Utf8Path) -> Result<ImageMetadata> {
    let bytes = filesystem::read(path)?;

    let mut reader = png::Decoder::new(io::Cursor::new(bytes))
      .read_info()
      .context(error::ImageDecodePng { path })?;

    reader.finish().context(error::ImageDecodePng { path })?;

    let info = reader.info();

    let mut title = None;

    for chunk in &info.uncompressed_latin1_text {
      if chunk.keyword == "Title" {
        ensure!(title.is_none(), error::ImageTitleMultiple { path });
        title = Some(chunk.text.clone());
      }
    }

    for chunk in &info.utf8_text {
      if chunk.keyword == "Title" {
        ensure!(title.is_none(), error::ImageTitleMultiple { path });
        title = Some(chunk.get_text().context(error::ImageDecodePng { path })?);
      }
    }

    let title = Self::title(title, path)?;

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
      title,
    })
  }

  pub(crate) fn oriented_dimensions(&self) -> Dimensions {
    self.orientation.dimensions(self.dimensions)
  }

  pub(crate) fn thumbnail_path(&self, ty: ImageType) -> Result<RelativePath> {
    let path = format!("{}.{}", self.thumbnail_stem(), ty.extension());
    path.parse().context(error::Path { path: &path })
  }

  pub(crate) fn thumbnail_stem(&self) -> String {
    format!("{}/{}", Self::THUMBNAIL_DIR, self.path.stem())
  }

  fn title(title: Option<String>, path: &Utf8Path) -> Result<Option<Text>> {
    let Some(title) = title else {
      return Ok(None);
    };

    Ok(Some(
      title
        .parse::<Text>()
        .context(error::ImageTitleInvalid { path })?,
    ))
  }
}

impl Content for Image {
  type Type = ImageType;

  fn info(&self, builder: InfoBuilder) -> InfoBuilder {
    builder
      .value("type", self.ty)
      .value("dimensions", self.dimensions)
      .value("orientation", self.orientation)
      .value("color type", self.color_type)
      .value("bit depth", format!("{}-bit", self.bit_depth))
      .optional("chroma subsampling", self.chroma_subsampling)
      .value("alpha", self.alpha)
  }

  fn load(root: &Utf8Path, path: RelativePath) -> Result<Item<Self>> {
    let ty = ImageType::from_path(&path).context(error::Path { path: &path })?;

    let ImageMetadata {
      alpha,
      bit_depth,
      chroma_subsampling,
      color_type,
      dimensions,
      orientation,
      title,
    } = match ty {
      ImageType::Jpeg => Self::decode_jpeg(&root.join(&path))?,
      ImageType::Png => Self::decode_png(&root.join(&path))?,
    };

    Ok(Item {
      content: Self {
        alpha,
        bit_depth,
        chroma_subsampling,
        color_type,
        dimensions,
        orientation,
        path,
        ty,
      },
      title,
    })
  }

  fn path(&self) -> &RelativePath {
    &self.path
  }

  #[cfg(test)]
  fn test(path: &str) -> Self {
    let path = path.parse::<RelativePath>().unwrap();
    let ty = ImageType::from_path(&path).unwrap();
    Self {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: None,
      color_type: ColorType::Rgb,
      dimensions: Dimensions {
        height: 1,
        width: 1,
      },
      orientation: Orientation::new(),
      path,
      ty,
    }
  }

  fn ty(&self) -> Self::Type {
    self.ty
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    ::image::{ImageEncoder, ImageFormat, codecs::jpeg::JpegEncoder},
  };

  #[test]
  fn create_thumbnail() {
    #[track_caller]
    fn case(source: (u32, u32), expected: (u32, u32)) {
      let (_tempdir, root) = tempdir();

      gradient(source.0, source.1)
        .save_with_format(root.join("foo.png"), ImageFormat::Png)
        .unwrap();

      let destination = Image::test("foo.png")
        .create_thumbnail(&root)
        .unwrap()
        .unwrap();

      assert_eq!(destination, "thumbnails/foo.jpg");

      let thumbnail = ::image::open(root.join(&destination)).unwrap();

      assert_eq!((thumbnail.width(), thumbnail.height()), expected);
    }

    case((1280, 640), (1024, 512));
    case((640, 1280), (512, 1024));
    case((640, 320), (640, 320));
  }

  #[test]
  fn create_thumbnail_alpha() {
    #[track_caller]
    fn case(alpha: u8, expected: &str) {
      let (_tempdir, root) = tempdir();

      gradient_alpha(1280, 640, alpha)
        .save_with_format(root.join("foo.png"), ImageFormat::Png)
        .unwrap();

      let destination = Image::test("foo.png")
        .create_thumbnail(&root)
        .unwrap()
        .unwrap();

      assert_eq!(destination, expected);

      let thumbnail = ::image::open(root.join(&destination)).unwrap();

      assert_eq!((thumbnail.width(), thumbnail.height()), (1024, 512));

      assert_eq!(thumbnail.color().has_alpha(), alpha < u8::MAX);

      if alpha < u8::MAX {
        assert!(thumbnail.to_rgba8().pixels().all(|pixel| pixel[3] == alpha));
      }
    }

    case(128, "thumbnails/foo.png");
    case(255, "thumbnails/foo.jpg");
  }

  #[test]
  fn create_thumbnail_orientation() {
    let (_tempdir, root) = tempdir();

    let mut encoded = Vec::new();

    let mut encoder = JpegEncoder::new(&mut encoded);

    encoder.set_exif_metadata(exif(6)).unwrap();

    encoder.encode_image(&gradient(2048, 1024)).unwrap();

    std::fs::write(root.join("foo.jpg"), encoded).unwrap();

    let destination = Image::test("foo.jpg")
      .create_thumbnail(&root)
      .unwrap()
      .unwrap();

    let thumbnail = ::image::open(root.join(&destination)).unwrap();

    assert_eq!((thumbnail.width(), thumbnail.height()), (512, 1024));
  }

  #[test]
  fn create_thumbnail_skips_larger() {
    let (_tempdir, root) = tempdir();

    gradient(1, 1)
      .save_with_format(root.join("foo.png"), ImageFormat::Png)
      .unwrap();

    assert_eq!(
      Image::test("foo.png").create_thumbnail(&root).unwrap(),
      None,
    );

    assert!(!root.join("thumbnails/foo.jpg").exists());
  }

  #[test]
  fn load() {
    #[track_caller]
    fn case(filename: &str, bytes: &[u8]) -> Result<Image> {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(filename), bytes).unwrap();

      Image::load(&root, filename.parse().unwrap()).map(|item| item.content)
    }

    #[track_caller]
    fn title(filename: &str, bytes: &[u8]) -> Result<Option<Text>> {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join(filename), bytes).unwrap();

      Image::load(&root, filename.parse().unwrap()).map(|item| item.title)
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

    assert_eq!(
      title("foo.png", &PngBuilder::new().text("Title", "bar").build()).unwrap(),
      Some("bar".parse().unwrap()),
    );

    assert_eq!(
      title("foo.png", &PngBuilder::new().itxt("Title", "bär").build()).unwrap(),
      Some("bär".parse().unwrap()),
    );

    assert_eq!(
      title(
        "foo.png",
        &PngBuilder::new().trailing_text("Title", "bar").build(),
      )
      .unwrap(),
      Some("bar".parse().unwrap()),
    );

    assert_eq!(
      title("foo.png", &PngBuilder::new().ztxt("Title", "bar").build()).unwrap(),
      None,
    );

    assert_eq!(
      title("foo.png", &PngBuilder::new().text("Foo", "bar").build()).unwrap(),
      None,
    );

    assert_matches_regex!(
      title(
        "foo.png",
        &PngBuilder::new()
          .text("Title", "bar")
          .itxt("Title", "baz")
          .build(),
      )
      .unwrap_err()
      .to_string(),
      r"^multiple titles in image `.*foo\.png`$",
    );

    assert_matches_regex!(
      title("foo.png", &PngBuilder::new().text("Title", "").build())
        .unwrap_err()
        .to_string(),
      r"^invalid title in image `.*foo\.png`$",
    );

    assert_matches_regex!(
      title("foo.png", &PngBuilder::new().text("Title", "\u{1}").build())
        .unwrap_err()
        .to_string(),
      r"^invalid title in image `.*foo\.png`$",
    );

    assert_eq!(
      title(
        "foo.jpg",
        &JpegBuilder::new()
          .xmp(&xmp::packet(&[("x-default", "bar")]))
          .build(),
      )
      .unwrap(),
      Some("bar".parse().unwrap()),
    );

    assert_eq!(title("foo.jpg", &JpegBuilder::new().build()).unwrap(), None,);

    assert_matches_regex!(
      title("foo.jpg", &JpegBuilder::new().xmp(b"<foo").build())
        .unwrap_err()
        .to_string(),
      r"^invalid XMP in image `.*foo\.jpg`$",
    );
  }

  #[test]
  fn oriented_dimensions() {
    let mut image = Image::test("foo.png");

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

  #[test]
  fn thumbnail_path() {
    #[track_caller]
    fn case(ty: ImageType, expected: &str) {
      assert_eq!(
        Image::test("foo/bar baz.png").thumbnail_path(ty).unwrap(),
        expected,
      );
    }

    case(ImageType::Jpeg, "thumbnails/bar baz.jpg");
    case(ImageType::Png, "thumbnails/bar baz.png");
  }

  #[test]
  fn thumbnail_stem() {
    assert_eq!(
      Image::test("foo/bar baz.png").thumbnail_stem(),
      "thumbnails/bar baz",
    );
  }
}
