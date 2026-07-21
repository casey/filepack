use super::*;

#[derive(
  Clone, Debug, Decode, DeserializeFromStr, Encode, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay,
)]
#[cbor(transparent, validate)]
pub struct ComponentBuf(String);

impl ComponentBuf {
  pub(crate) fn from_component(component: &Component) -> Self {
    Self(component.as_str().to_owned())
  }

  pub(crate) fn from_integer(i: usize) -> Self {
    i.to_string().parse().unwrap()
  }
}

impl AsRef<str> for ComponentBuf {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl AsRef<Utf8Path> for ComponentBuf {
  fn as_ref(&self) -> &Utf8Path {
    self.0.as_ref()
  }
}

impl Borrow<Component> for ComponentBuf {
  fn borrow(&self) -> &Component {
    self
  }
}

impl Borrow<str> for ComponentBuf {
  fn borrow(&self) -> &str {
    &self.0
  }
}

impl Deref for ComponentBuf {
  type Target = Component;

  fn deref(&self) -> &Component {
    Component::from_component_buf(self)
  }
}

impl Display for ComponentBuf {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<&Self> for ComponentBuf {
  fn from(component: &Self) -> Self {
    component.clone()
  }
}

impl FromStr for ComponentBuf {
  type Err = ComponentError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from_component(Component::new(s)?))
  }
}

impl PartialEq<&str> for ComponentBuf {
  fn eq(&self, s: &&str) -> bool {
    self.as_str().eq(*s)
  }
}

impl Validate for ComponentBuf {
  fn validate(&self) -> Result<(), DecodeError> {
    Component::new(&self.0).context(decode_error::Component)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor("foo".parse::<ComponentBuf>().unwrap(), "63666f6f");
    let empty = "".encode_to_vec();
    let mut decoder = Decoder::new(&empty);
    assert_matches!(
      ComponentBuf::decode(&mut decoder),
      Err(DecodeError::Component {
        source: ComponentError::Empty
      }),
    );
  }
}
