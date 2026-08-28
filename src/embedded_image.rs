use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct EmbeddedImage {
  pub(crate) data: Vec<u8>,
  pub(crate) media_type: Mime,
}

impl EmbeddedImage {
  pub(crate) fn dimensions(&self) -> Result<Dimensions, AudioError> {
    if self.media_type == mime::IMAGE_JPEG {
      let mut decoder = JpegDecoder::new(io::Cursor::new(&self.data));

      decoder
        .decode_headers()
        .context(audio_error::EmbeddedImageDecodeJpeg)?;

      let info = decoder.info().unwrap();

      Ok(Dimensions {
        height: info.height.into(),
        width: info.width.into(),
      })
    } else if self.media_type == mime::IMAGE_PNG {
      let reader = png::Decoder::new(io::Cursor::new(&self.data))
        .read_info()
        .context(audio_error::EmbeddedImageDecodePng)?;

      let info = reader.info();

      Ok(Dimensions {
        height: info.height.into(),
        width: info.width.into(),
      })
    } else {
      Err(
        audio_error::EmbeddedImageMediaType {
          media_type: self.media_type.clone(),
        }
        .build(),
      )
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dimensions() {
    #[track_caller]
    fn case(media_type: &str, data: Vec<u8>, width: u64, height: u64) {
      assert_eq!(
        EmbeddedImage {
          data,
          media_type: media_type.parse().unwrap()
        }
        .dimensions()
        .unwrap(),
        Dimensions { height, width },
      );
    }

    #[track_caller]
    fn err(media_type: &str, data: &[u8]) -> AudioError {
      EmbeddedImage {
        data: data.into(),
        media_type: media_type.parse().unwrap(),
      }
      .dimensions()
      .unwrap_err()
    }

    case(
      "image/png",
      PngBuilder::new().width(2).height(3).build(),
      2,
      3,
    );
    case(
      "image/jpeg",
      JpegBuilder::new().width(4).height(5).build(),
      4,
      5,
    );

    assert_matches!(
      err("image/png", b"foo"),
      AudioError::EmbeddedImageDecodePng { .. }
    );
    assert_matches!(
      err("image/jpeg", b"foo"),
      AudioError::EmbeddedImageDecodeJpeg { .. }
    );
    assert_matches!(
      err("image/jpg", &PngBuilder::new().build()),
      AudioError::EmbeddedImageMediaType { media_type } if media_type == "image/jpg"
    );
  }
}
