use super::*;

pub trait Message: DecodeOwned + Encode + Eq + Ord {
  const CONTEXT: Context;
  const TAG: Tag;
  type Error: From<SignatureError>;
  type Policy<'a>;

  fn check(&self, signer: PublicKey, policy: Self::Policy<'_>) -> Result<(), Self::Error>;

  fn digest(&self, version: Version) -> Hash {
    let envelope = Envelope {
      version,
      application: Application::Filepack,
      context: Self::CONTEXT,
      message: self,
    };
    Hash::bytes(&envelope.encode_to_vec())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::any::TypeId};

  #[test]
  fn context_separates_domains() {
    #[allow(clippy::arbitrary_source_item_ordering)]
    #[derive(Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    #[deco(strict)]
    struct Impostor {
      #[n(0)]
      version: Version,
      #[n(1)]
      fingerprint: Fingerprint,
      #[n(2)]
      timestamp: Option<u64>,
    }

    impl Message for Impostor {
      const CONTEXT: Context = Context::Claims;
      const TAG: Tag = Tag::Token;
      type Error = AuthorizationError;
      type Policy<'a> = ();

      fn check(&self, _signer: PublicKey, _policy: ()) -> Result<(), AuthorizationError> {
        Ok(())
      }
    }

    #[allow(clippy::arbitrary_source_item_ordering)]
    #[derive(Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    #[deco(strict)]
    struct Twin {
      #[n(0)]
      version: Version,
      #[n(1)]
      fingerprint: Fingerprint,
      #[n(2)]
      timestamp: Option<u64>,
    }

    impl Message for Twin {
      const CONTEXT: Context = Context::Statement;
      const TAG: Tag = Tag::Signature;
      type Error = Error;
      type Policy<'a> = ();

      fn check(&self, _signer: PublicKey, _policy: ()) -> Result {
        Ok(())
      }
    }

    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    assert_eq!(
      private_key
        .sign(Twin {
          version: Version::Zero,
          fingerprint,
          timestamp: None,
        })
        .to_string(),
      test::SIGNATURE,
    );

    assert_matches!(
      test::SIGNATURE
        .replacen("signature1", "token1", 1)
        .parse::<Signature<Impostor>>()
        .unwrap()
        .verify(())
        .unwrap_err(),
      AuthorizationError::Signature {
        source: SignatureError::Invalid { public_key: signer, .. },
      } if signer == public_key,
    );
  }

  #[test]
  fn domain_separators() {
    #[derive(Default)]
    struct Cases {
      contexts: BTreeSet<Context>,
      types: BTreeSet<TypeId>,
    }

    impl Cases {
      #[track_caller]
      fn case<T: Message + 'static>(&mut self, message: T, context: u64) {
        let mut encoder = Encoder::new();

        let mut map = encoder.map::<u64>();
        map.item(3, &message);
        map.item(2, context);
        map.item(1, "filepack");
        map.item(0, Version::Zero);
        map.finish();

        assert_eq!(
          message.digest(Version::Zero),
          Hash::bytes(&encoder.finish())
        );

        self.contexts.insert(T::CONTEXT);
        self.types.insert(TypeId::of::<T>());
      }
    }

    let mut cases = Cases::default();

    cases.case(
      Claims {
        version: Version::Zero,
        audience: "foo".into(),
        timestamp: 1000,
      },
      0,
    );

    cases.case(
      Statement {
        version: Version::Zero,
        fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
        timestamp: Some(1000),
      },
      1,
    );

    assert_eq!(cases.contexts, Context::iter().collect());
    assert_eq!(cases.types.len(), cases.contexts.len());
  }
}
