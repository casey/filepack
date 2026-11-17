use super::*;

/// BLAKE3 hash of a revision.
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
#[deco(transparent)]
pub struct RevisionHash(pub(crate) Hash);

impl RevisionHash {
  pub(crate) const LEN: usize = Hash::LEN;
  #[cfg(test)]
  pub(crate) const ZERO: Self = Self(Hash::ZERO);

  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }

  pub(crate) fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
    Self(bytes.into())
  }
}

impl Display for RevisionHash {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.format(f)
  }
}

impl Hex for RevisionHash {
  const EXAMPLE: &'static str =
    "revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583";
  const EXAMPLE_DESCRIPTION: &'static str = "Hash of the revision with no parents whose package \
    is the empty package.";
  const TAG: Tag = Tag::Revision;
}

impl Message for RevisionHash {
  const CONTEXT: Context = Context::Revision;
  const NAME: &'static str = "RevisionHash";
  const SIGNATURE_EXAMPLE: &'static str = concat!(
    "signature101a03b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2902a02c7922354e",
    "5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b4458303c0b0917f33a73bc9a24cb5b04df40442113e",
    "0798e8e7ecad55b14ca2fe665a1f02dd658d2ea50d4b848461dd2d06f4905113e800511cb30090354491690ada41",
    "06",
  );
  const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str = "Signature over the hash of the revision \
    with no parents whose package is the empty package by the all-zero private key.";
  const SIGNATURE_TAG: Tag = Tag::Signature;
  type Error = Error;
  type Policy<'a> = RevisionHash;

  fn check(&self, _signer: PublicKey, revision: RevisionHash) -> Result {
    ensure! {
      *self == revision,
      error::SignatureRevisionMismatch {
        package: revision,
        signature: *self,
      },
    }
    Ok(())
  }
}

impl FromStr for RevisionHash {
  type Err = HexError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl From<[u8; Self::LEN]> for RevisionHash {
  fn from(bytes: [u8; Self::LEN]) -> Self {
    Self::from_bytes(bytes)
  }
}

impl From<Hash> for RevisionHash {
  fn from(hash: Hash) -> Self {
    Self(hash)
  }
}

impl json::Json for RevisionHash {
  fn json_schema(registry: &mut json::Registry) -> String {
    json::Schema::Hex {
      description: "BLAKE3 hash of a revision.".into(),
      tag: Tag::Revision,
    }
    .register("RevisionHash", registry)
  }
}

impl redb::Key for RevisionHash {
  fn compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
  }
}

impl redb::Value for RevisionHash {
  type AsBytes<'a>
    = &'a [u8; Self::LEN]
  where
    Self: 'a;

  type SelfType<'a>
    = RevisionHash
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
    TypeName::Revision.into()
  }
}

impl PathExtractor for RevisionHash {
  fn parameters() -> Vec<reference::Parameter> {
    vec![reference::Parameter::RevisionHash]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn example() {
    assert_eq!(
      PrivateKey::zero()
        .sign(RevisionHash::EXAMPLE.parse::<RevisionHash>().unwrap())
        .to_string(),
      Attestation::EXAMPLE,
    );
  }

  #[test]
  fn mismatch() {
    let revision = RevisionHash::EXAMPLE.parse::<RevisionHash>().unwrap();
    let other = RevisionHash::ZERO;
    assert_matches!(
      Attestation::EXAMPLE
        .parse::<Attestation>()
        .unwrap()
        .verify(other)
        .unwrap_err(),
      Error::SignatureRevisionMismatch {
        package,
        signature,
        ..
      } if package == other && signature == revision,
    );
  }

  #[test]
  fn redb() {
    assert_redb_impls(&[
      RevisionHash::ZERO,
      RevisionHash::from(Hash::from([0xff; Hash::LEN])),
      RevisionHash::from(Hash::bytes(b"foo")),
      RevisionHash::from(Hash::bytes(b"bar")),
    ]);
  }
}
