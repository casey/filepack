use super::*;

#[derive(Boilerplate)]
pub(crate) struct InfoHtml<'a>(pub(crate) &'a Info);

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn code() {
    assert_eq!(
      InfoHtml(&Info::Code("foo".into())).to_string(),
      "<code>foo</code>\n",
    );
  }

  #[test]
  fn code_link() {
    assert_eq!(
      InfoHtml(&Info::Link {
        code: true,
        text: "foo".into(),
        url: "/bar".into(),
      })
      .to_string(),
      "<a href='/bar'><code>foo</code></a>\n",
    );
  }

  #[test]
  fn link() {
    assert_eq!(
      InfoHtml(&Info::Link {
        code: false,
        text: "foo".into(),
        url: "/bar".into(),
      })
      .to_string(),
      "<a href='/bar'>foo</a>\n",
    );
  }

  #[test]
  fn list() {
    assert_eq!(
      InfoHtml(&Info::List(vec![
        Info::Value("foo".into()),
        Info::Map(vec![("bar".into(), Info::Value("baz".into()))]),
      ]))
      .to_string(),
      unindent(
        "
          <ol role=list>
            <li>
              foo
            </li>
            <li>
              <dl>
                <div>
                  <dt>bar</dt>
                  <dd>
                    baz
                  </dd>
                </div>
              </dl>
            </li>
          </ol>
        "
      ),
    );
  }

  #[test]
  fn map() {
    assert_eq!(
      InfoHtml(&Info::Map(vec![
        ("foo".into(), Info::Value("bar".into())),
        (
          "baz".into(),
          Info::Map(vec![("qux".into(), Info::Value("quux".into()))]),
        ),
      ]))
      .to_string(),
      unindent(
        "
          <dl>
            <div>
              <dt>foo</dt>
              <dd>
                bar
              </dd>
            </div>
            <div>
              <dt>baz</dt>
              <dd>
                <dl>
                  <div>
                    <dt>qux</dt>
                    <dd>
                      quux
                    </dd>
                  </div>
                </dl>
              </dd>
            </div>
          </dl>
        "
      ),
    );
  }

  #[test]
  fn value() {
    assert_eq!(InfoHtml(&Info::Value("foo".into())).to_string(), "foo\n");
  }
}
