use super::*;

pub(crate) struct InfoBuilder {
  map: Vec<(String, Info)>,
}

impl InfoBuilder {
  pub(crate) fn build(self) -> Info {
    Info::Map(self.map)
  }

  pub(crate) fn link(mut self, key: &str, text: impl Display, url: String) -> Self {
    self.map.push((
      key.into(),
      Info::Link {
        text: text.to_string(),
        url,
      },
    ));
    self
  }

  pub(crate) fn list(mut self, key: &str, values: impl IntoIterator<Item = Info>) -> Self {
    self
      .map
      .push((key.into(), Info::List(values.into_iter().collect())));
    self
  }

  pub(crate) fn new() -> Self {
    Self { map: Vec::new() }
  }

  pub(crate) fn optional(self, key: &str, value: Option<impl Display>) -> Self {
    if let Some(value) = value {
      self.value(key, value)
    } else {
      self
    }
  }

  pub(crate) fn value(mut self, key: &str, value: impl Display) -> Self {
    self.map.push((key.into(), Info::Value(value.to_string())));
    self
  }

  pub(crate) fn when(self, condition: bool, key: &str, value: impl Display) -> Self {
    if condition {
      self.value(key, value)
    } else {
      self
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn build() {
    assert_eq!(
      InfoBuilder::new()
        .value("foo", "bar")
        .optional("baz", Some("qux"))
        .optional("quux", None::<&str>)
        .when(true, "plugh", "xyzzy")
        .when(false, "thud", "wibble")
        .link("corge", "grault", "garply".into())
        .list("waldo", [Info::Value("fred".into())])
        .build(),
      Info::Map(vec![
        ("foo".into(), Info::Value("bar".into())),
        ("baz".into(), Info::Value("qux".into())),
        ("plugh".into(), Info::Value("xyzzy".into())),
        (
          "corge".into(),
          Info::Link {
            text: "grault".into(),
            url: "garply".into(),
          },
        ),
        ("waldo".into(), Info::List(vec![Info::Value("fred".into())])),
      ]),
    );
  }
}
