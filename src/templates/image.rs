use super::*;

#[derive(Boilerplate)]
pub(crate) struct ImageHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) image: usize,
  pub(crate) metadata: Metadata,
}

impl ImageHtml {
  fn image(&self) -> &Image {
    let Media::Image { items } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &items[self.image]
  }
}

impl Page for ImageHtml {
  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    Some(OpenGraphImage {
      dimensions: self.image().oriented_dimensions(),
      path: format!(
        "media/image/{}/item/{}",
        self.fingerprint,
        Ordinal(self.image)
      ),
    })
  }

  fn script(&self) -> Option<&'static str> {
    Some("/static/image.js")
  }

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
  fn open_graph_image() {
    let html = ImageHtml {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      image: 0,
      metadata: Metadata {
        media: Some(Media::Image {
          items: vec![Image {
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
          }],
        }),
        ..default()
      },
    };

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 2,
          width: 1,
        },
        path: format!("media/image/{}/item/1", test::FINGERPRINT),
      }),
    );
  }

  #[test]
  fn oriented_dimensions() {
    assert_eq!(
      ImageHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        image: 0,
        metadata: Metadata {
          media: Some(Media::Image {
            items: vec![Image {
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
