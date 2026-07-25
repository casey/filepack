use super::*;

#[derive(Boilerplate)]
pub(crate) struct ImageHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) image: usize,
  pub(crate) metadata: Metadata,
}

impl ImageHtml {
  fn image(&self) -> &Image {
    let Media::Image { images } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &images[self.image]
  }
}

impl Page for ImageHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/image.css")
  }

  fn title(&self) -> String {
    format!("image {} · filepack", self.image)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn oriented_dimensions() {
    assert_eq!(
      ImageHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        image: 0,
        metadata: Metadata {
          media: Some(Media::Image {
            images: vec![Image {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              filename: "foo.png".parse().unwrap(),
              orientation: Orientation {
                mirrored: false,
                rotation: Rotation::R90,
              },
              ty: ImageType::Png,
            }],
          }),
          ..default()
        },
      }
      .to_string(),
      format!(
        "<img src=/media/image/{}/item/1 width=1 height=2>\n",
        test::FINGERPRINT,
      ),
    );
  }
}
