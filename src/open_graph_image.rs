use super::*;

#[derive(Debug, PartialEq)]
pub struct OpenGraphImage {
  pub(crate) dimensions: Dimensions,
  pub(crate) path: String,
}

impl OpenGraphImage {
  pub(crate) fn artwork(metadata: &Metadata, fingerprint: Fingerprint) -> Option<Self> {
    Some(Self::thumbnail(
      metadata,
      metadata.artwork.as_ref()?,
      format!("artwork/{fingerprint}"),
    ))
  }

  pub(crate) fn thumbnail(metadata: &Metadata, image: &Image, path: String) -> Self {
    if let Some(thumbnail) = metadata.thumbnail(&image.path) {
      Self {
        dimensions: thumbnail.oriented_dimensions(),
        path: format!("{path}/thumbnail"),
      }
    } else {
      Self {
        dimensions: image.oriented_dimensions(),
        path,
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn thumbnail() {
    #[track_caller]
    fn case(metadata: &Metadata, expected: OpenGraphImage) {
      assert_eq!(
        OpenGraphImage::thumbnail(metadata, metadata.artwork.as_ref().unwrap(), "foo".into()),
        expected,
      );
    }

    let artwork = Image {
      alpha: false,
      bit_depth: 8,
      chroma_subsampling: None,
      color_type: ColorType::Rgb,
      dimensions: Dimensions {
        height: 1,
        width: 2,
      },
      orientation: Orientation {
        mirrored: false,
        rotation: Rotation::R90,
      },
      path: "foo.png".parse().unwrap(),
      ty: ImageType::Png,
    };

    case(
      &Metadata {
        artwork: Some(artwork.clone()),
        ..default()
      },
      OpenGraphImage {
        dimensions: Dimensions {
          height: 2,
          width: 1,
        },
        path: "foo".into(),
      },
    );

    case(
      &Metadata {
        artwork: Some(artwork),
        thumbnails: Some(
          [(
            "foo.png".parse().unwrap(),
            Image {
              alpha: false,
              bit_depth: 8,
              chroma_subsampling: None,
              color_type: ColorType::Rgb,
              dimensions: Dimensions {
                height: 3,
                width: 4,
              },
              orientation: Orientation::default(),
              path: "thumbnails/foo.jpg".parse().unwrap(),
              ty: ImageType::Jpeg,
            },
          )]
          .into(),
        ),
        ..default()
      },
      OpenGraphImage {
        dimensions: Dimensions {
          height: 3,
          width: 4,
        },
        path: "foo/thumbnail".into(),
      },
    );
  }
}
