use super::*;

pub trait Message: DecodeOwned + Encode + Eq + Ord {
  const CONTEXT: Context;
  const NAME: &'static str;
  const SIGNATURE_EXAMPLE: &'static str;
  const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str;
  const SIGNATURE_TAG: Tag;
  type Error: From<SignatureError>;
  type Policy<'a>;

  fn check(&self, signer: PublicKey, policy: Self::Policy<'_>) -> Result<(), Self::Error>;

  fn digest(&self) -> Hash {
    let envelope = Envelope {
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
    /// Impostor.
    #[derive(Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    #[deco(transparent)]
    struct Impostor(Hash);

    impl Message for Impostor {
      const CONTEXT: Context = Context::Claims;
      const NAME: &'static str = "Impostor";
      const SIGNATURE_EXAMPLE: &'static str = "";
      const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str = "";
      const SIGNATURE_TAG: Tag = Tag::Token;
      type Error = TokenError;
      type Policy<'a> = ();

      fn check(&self, _signer: PublicKey, _policy: ()) -> Result<(), TokenError> {
        Ok(())
      }
    }

    /// Twin.
    #[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    #[deco(transparent)]
    struct Twin(Hash);

    impl Message for Twin {
      const CONTEXT: Context = Context::Revision;
      const NAME: &'static str = "Twin";
      const SIGNATURE_EXAMPLE: &'static str = "";
      const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str = "";
      const SIGNATURE_TAG: Tag = Tag::Signature;
      type Error = AttestationError;
      type Policy<'a> = ();

      fn check(&self, _signer: PublicKey, _policy: ()) -> Result<(), AttestationError> {
        Ok(())
      }
    }

    let private_key = DisplayPrivateKey::EXAMPLE.parse::<PrivateKey>().unwrap();
    let public_key = PublicKey::EXAMPLE.parse::<PublicKey>().unwrap();
    let revision = RevisionHash::EXAMPLE.parse::<RevisionHash>().unwrap();

    assert_eq!(
      private_key.sign(Twin(revision.0)).to_string(),
      Attestation::EXAMPLE,
    );

    assert_matches!(
      Attestation::EXAMPLE
        .replacen("signature1", "token1", 1)
        .parse::<Signature<Impostor>>()
        .unwrap()
        .verify(())
        .unwrap_err(),
      TokenError::Signature {
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
        map.finish();

        assert_eq!(message.digest(), Hash::bytes(&encoder.finish()));

        self.contexts.insert(T::CONTEXT);
        self.types.insert(TypeId::of::<T>());
      }
    }

    let mut cases = Cases::default();

    cases.case(RevisionHash::ZERO, 0);

    cases.case(
      Claims {
        domain: "foo".into(),
        timestamp: 1000,
      },
      1,
    );

    assert_eq!(cases.contexts, Context::iter().collect());
    assert_eq!(cases.types.len(), cases.contexts.len());
  }
}
