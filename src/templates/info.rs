use super::*;

#[derive(Boilerplate)]
pub(crate) struct InfoHtml<'a>(pub(crate) &'a Info);

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

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
          <ol>
            <li>
              foo
            </li>
            <li>
              <dl>
                <dt>bar</dt>
                <dd>
                  baz
                </dd>
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
            <dt>foo</dt>
            <dd>
              bar
            </dd>
            <dt>baz</dt>
            <dd>
              <dl>
                <dt>qux</dt>
                <dd>
                  quux
                </dd>
              </dl>
            </dd>
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
