use super::*;

#[derive(Boilerplate)]
pub(crate) struct MediaHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) identifier: PackageIdentifier,
  pub(crate) metadata: Metadata,
  pub(crate) number: Option<u64>,
}

impl MediaHtml {
  fn info(&self) -> Info {
    InfoBuilder::new()
      .when_some(self.number, |builder, number| {
        builder.link("number", number, format!("/package/{number}"))
      })
      .code_link(
        "fingerprint",
        self.fingerprint,
        format!("/package/{}", self.fingerprint),
      )
      .list(
        "items",
        self
          .metadata
          .media
          .as_ref()
          .unwrap()
          .items()
          .enumerate()
          .map(|(i, item)| item.info(format!("/package/{}/item/{}", self.identifier, Ordinal(i)))),
      )
      .build()
  }

  fn title(&self) -> Option<&str> {
    self.metadata.title.as_deref()
  }
}

impl Page for MediaHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/media.css")
  }

  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} media · Filepack")
    } else {
      format!("{} media · Filepack", self.fingerprint)
    }
  }

  fn up(&self) -> Option<String> {
    Some(format!("/package/{}", self.identifier))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn media() {
    for identifier in [
      PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
      PackageIdentifier::Number(1),
    ] {
      assert_eq!(
        MediaHtml {
          fingerprint: test::FINGERPRINT.parse().unwrap(),
          identifier,
          metadata: Metadata {
            media: Some(Media::Image {
              items: vec![Item::test("foo.png")],
            }),
            ..default()
          },
          number: Some(1),
        }
        .to_string(),
        unindent(&format!(
          "
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>items</dt>
              <dd>
                <ol role=list>
                  <li>
                    <dl>
                      <div>
                        <dt>file</dt>
                        <dd>
                          <a href='/package/{identifier}/item/1'>foo.png</a>
                        </dd>
                      </div>
                      <div>
                        <dt>type</dt>
                        <dd>
                          PNG
                        </dd>
                      </div>
                      <div>
                        <dt>dimensions</dt>
                        <dd>
                          1×1
                        </dd>
                      </div>
                      <div>
                        <dt>orientation</dt>
                        <dd>
                          0°
                        </dd>
                      </div>
                      <div>
                        <dt>color type</dt>
                        <dd>
                          RGB
                        </dd>
                      </div>
                      <div>
                        <dt>bit depth</dt>
                        <dd>
                          8-bit
                        </dd>
                      </div>
                      <div>
                        <dt>alpha</dt>
                        <dd>
                          false
                        </dd>
                      </div>
                      <div>
                        <dt>compression</dt>
                        <dd>
                          lossless
                        </dd>
                      </div>
                    </dl>
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
        ",
          fingerprint = test::FINGERPRINT,
        )),
      );
    }
  }

  #[test]
  fn up() {
    for identifier in [
      PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
      PackageIdentifier::Number(1),
    ] {
      assert_eq!(
        MediaHtml {
          fingerprint: test::FINGERPRINT.parse().unwrap(),
          identifier,
          metadata: default(),
          number: Some(1),
        }
        .up(),
        Some(format!("/package/{identifier}")),
      );
    }
  }
}
