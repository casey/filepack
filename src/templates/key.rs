use super::*;

#[derive(Boilerplate)]
pub(crate) struct KeyHtml {
  pub(crate) key: PublicKey,
  pub(crate) numbers: BTreeSet<u64>,
  pub(crate) revisions: BTreeSet<RevisionHash>,
}

impl KeyHtml {
  fn info(&self) -> Info {
    InfoBuilder::new()
      .when(!self.numbers.is_empty(), |builder| {
        builder.list(
          "packages",
          self
            .numbers
            .iter()
            .map(|number| Info::Link(Link::new(number).href(format!("/package/{number}")))),
        )
      })
      .list(
        "revisions",
        self.revisions.iter().map(|revision| {
          Info::Link(
            Link::new(revision)
              .href(format!("/package/{revision}"))
              .code(),
          )
        }),
      )
      .build()
  }
}

impl Page for KeyHtml {
  fn title(&self) -> String {
    format!("Key {} · Filepack", self.key)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn key() {
    assert_eq!(
      KeyHtml {
        key: PublicKey::EXAMPLE.parse().unwrap(),
        numbers: BTreeSet::from([1, 0]),
        revisions: BTreeSet::from([RevisionHash::EXAMPLE.parse().unwrap(), RevisionHash::ZERO]),
      }
      .to_string(),
      unindent(&format!(
        "
          <h1>Key {key}</h1>
          <dl>
            <div>
              <dt>packages</dt>
              <dd>
                <ol role=list>
                  <li>
                    <a href='/package/0'>0</a>
                  </li>
                  <li>
                    <a href='/package/1'>1</a>
                  </li>
                </ol>
              </dd>
            </div>
            <div>
              <dt>revisions</dt>
              <dd>
                <ol role=list>
                  <li>
                    <a href='/package/{zero}'><code>{zero}</code></a>
                  </li>
                  <li>
                    <a href='/package/{revision}'><code>{revision}</code></a>
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
        ",
        key = PublicKey::EXAMPLE,
        revision = RevisionHash::EXAMPLE,
        zero = RevisionHash::ZERO,
      )),
    );
  }
}
