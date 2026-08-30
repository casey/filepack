use super::*;

pub(crate) struct InfoBuilder {
  map: Vec<(String, Info)>,
}

impl InfoBuilder {
  pub(crate) fn build(self) -> Info {
    Info::Map(self.map)
  }

  pub(crate) fn code(mut self, key: &str, value: impl Display) -> Self {
    self.map.push((key.into(), Info::Code(value.to_string())));
    self
  }

  pub(crate) fn info(mut self, key: &str, info: Info) -> Self {
    self.map.push((key.into(), info));
    self
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

  pub(crate) fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
    if condition { f(self) } else { self }
  }

  pub(crate) fn when_some<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
    match value {
      Some(value) => f(self, value),
      None => self,
    }
  }

  pub(crate) fn with(self, f: impl FnOnce(Self) -> Self) -> Self {
    f(self)
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
        .link("corge", "grault", "garply".into())
        .list("waldo", [Info::Value("fred".into())])
        .code("wubble", "flob")
        .info(
          "spam",
          Info::Map(vec![("eggs".into(), Info::Value("ham".into()))])
        )
        .when_some(Some("bacon"), |builder, value| builder
          .value("toast", value))
        .when_some(None::<&str>, |builder, value| builder.value("jam", value))
        .when(true, |builder| builder.value("tea", "milk"))
        .when(false, |builder| builder.value("coffee", "sugar"))
        .with(|builder| builder.value("scone", "cream"))
        .build(),
      Info::Map(vec![
        ("foo".into(), Info::Value("bar".into())),
        ("baz".into(), Info::Value("qux".into())),
        (
          "corge".into(),
          Info::Link {
            text: "grault".into(),
            url: "garply".into(),
          },
        ),
        ("waldo".into(), Info::List(vec![Info::Value("fred".into())])),
        ("wubble".into(), Info::Code("flob".into())),
        (
          "spam".into(),
          Info::Map(vec![("eggs".into(), Info::Value("ham".into()))]),
        ),
        ("toast".into(), Info::Value("bacon".into())),
        ("tea".into(), Info::Value("milk".into())),
        ("scone".into(), Info::Value("cream".into())),
      ]),
    );
  }
}
