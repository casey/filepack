use super::*;

pub(crate) struct Attribute<T> {
  name: &'static str,
  value: Option<T>,
}

impl Attribute<&'static str> {
  pub(crate) fn class(value: Option<&'static str>) -> Trusted<Self> {
    Trusted(Self {
      name: "class",
      value,
    })
  }
}

impl Attribute<Language> {
  pub(crate) fn lang(value: Option<Language>) -> Trusted<Self> {
    Trusted(Self {
      name: "lang",
      value,
    })
  }
}

impl<T: Display> Display for Attribute<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use fmt::Write;

    if let Some(value) = &self.value {
      write!(f, " {}='", self.name)?;

      for c in value.to_string().chars() {
        match c {
          '&' => f.write_str("&amp;")?,
          '\'' => f.write_str("&apos;")?,
          c => f.write_char(c)?,
        }
      }

      f.write_str("'")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn class() {
    assert_eq!(Attribute::class(None).0.to_string(), "");
    assert_eq!(Attribute::class(Some("foo")).0.to_string(), " class='foo'");
  }

  #[test]
  fn escape() {
    assert_eq!(
      Attribute::class(Some("'&")).0.to_string(),
      " class='&apos;&amp;'",
    );
  }

  #[test]
  fn lang() {
    assert_eq!(Attribute::lang(None).0.to_string(), "");
    assert_eq!(
      Attribute::lang(Some("en".parse().unwrap())).0.to_string(),
      " lang='en'",
    );
  }
}
