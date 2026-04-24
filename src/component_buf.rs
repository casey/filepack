use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay)]
pub(crate) struct ComponentBuf(String);

impl ComponentBuf {
  pub(crate) fn from_component(component: &Component) -> Self {
    Self(component.as_str().to_owned())
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

impl Decode for ComponentBuf {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.text()?.parse().context(decode_error::Component)
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
    write!(f, "{}", &self.0)
  }
}

impl Encode for ComponentBuf {
  fn encode(&self, encoder: &mut Encoder) {
    self.0.as_str().encode(encoder);
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      "foo".parse::<ComponentBuf>().unwrap(),
      &[0x63, 0x66, 0x6f, 0x6f],
    );
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
