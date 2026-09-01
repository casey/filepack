use super::*;

#[derive(Boilerplate)]
pub(crate) struct MediaHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
}

impl MediaHtml {
  fn info(&self) -> Info {
    InfoBuilder::new()
      .code_link(
        "package",
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
          .map(|(i, item)| item.info(format!("/package/{}/item/{}", self.fingerprint, Ordinal(i)))),
      )
      .build()
  }

  fn title(&self) -> Option<&str> {
    self.metadata.title.as_deref()
  }
}

impl Page for MediaHtml {
  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} media · Filepack")
    } else {
      format!("{} media · Filepack", self.fingerprint)
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn media() {
    assert_eq!(
      MediaHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Metadata {
          media: Some(Media::Image {
            items: vec![Item::test("foo.png")],
          }),
          ..default()
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <dl>
            <div>
              <dt>package</dt>
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
                          <a href='/package/{fingerprint}/item/1'>foo.png</a>
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
