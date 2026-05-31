use super::*;

#[derive(
  Clone,
  Copy,
  Debug,
  Decode,
  DeserializeFromStr,
  Encode,
  Eq,
  Ord,
  PartialEq,
  PartialOrd,
  SerializeDisplay,
)]
#[cbor(transparent)]
pub struct Fingerprint(pub(crate) Hash);

impl Fingerprint {
  pub(crate) const LEN: usize = Hash::LEN;

  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }

  pub(crate) fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
    Self(bytes.into())
  }
}

impl Display for Fingerprint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32Encoder::new(Bech32Type::Fingerprint);
    encoder.bytes(self.as_bytes());
    write!(f, "{encoder}")
  }
}

impl FromStr for Fingerprint {
  type Err = Bech32Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let inner = Bech32Decoder::decode_byte_array(Bech32Type::Fingerprint, s)?;
    Ok(Self(inner.into()))
  }
}

impl From<[u8; Self::LEN]> for Fingerprint {
  fn from(bytes: [u8; Self::LEN]) -> Self {
    Self::from_bytes(bytes)
  }
}

impl From<Hash> for Fingerprint {
  fn from(hash: Hash) -> Self {
    Self(hash)
  }
}

impl redb::Key for Fingerprint {
  fn compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
  }
}

impl redb::Value for Fingerprint {
  type AsBytes<'a>
    = &'a [u8; Self::LEN]
  where
    Self: 'a;

  type SelfType<'a>
    = Fingerprint
  where
    Self: 'a;

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    value.as_bytes()
  }

  fn fixed_width() -> Option<usize> {
    Some(Self::LEN)
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    <[u8; Self::LEN]>::try_from(data).unwrap().into()
  }

  fn type_name() -> redb::TypeName {
    redb::TypeName::new("filepack-fingerprint")
  }
}
