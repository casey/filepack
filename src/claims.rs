use super::*;

const LEEWAY: u64 = 30;
const TTL: u64 = 60;

pub(crate) struct Policy<'a> {
  admin: PublicKey,
  domain: &'a Domain,
  now: u64,
}

/// Claims made by an authorization token.
#[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Claims {
  /// Domain the token is valid for.
  #[n(1)]
  pub(crate) domain: String,
  /// Unix timestamp of signing in seconds.
  #[n(2)]
  pub(crate) timestamp: u64,
}

impl Claims {
  pub(crate) fn sign(private_key: &PrivateKey, domain: &str) -> Result<String> {
    Ok(
      private_key
        .sign(Claims {
          domain: domain.into(),
          timestamp: now().context(error::Time)?,
        })
        .to_string(),
    )
  }

  pub(crate) fn verify(
    admin: PublicKey,
    domain: &Domain,
    now: u64,
    token: &str,
  ) -> Result<(), TokenError> {
    let signature = token.parse::<Token>().context(token_error::Token)?;

    let policy = Policy { admin, domain, now };

    signature.verify(policy)?;

    Ok(())
  }
}

impl Message for Claims {
  const CONTEXT: Context = Context::Claims;
  const NAME: &'static str = "Claims";
  const SIGNATURE_EXAMPLE: &'static str = concat!(
    "token101a03b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da290290018c66696c6570",
    "61636b2e636f6d020003c0c99690da16d960d5afa356c4d73f79eef22f07981adc74efbf975db9aa343152c810cb",
    "c95cde36d8c377cb99c475555f2a2db63568c0afb1e83c101b835c540e",
  );
  const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str =
    "Token with domain `filepack.com` and timestamp 0, signed by the all-zero private key.";
  const SIGNATURE_TAG: Tag = Tag::Token;
  type Error = TokenError;
  type Policy<'a> = Policy<'a>;

  fn check(&self, signer: PublicKey, policy: Policy) -> Result<(), TokenError> {
    ensure! {
      signer == policy.admin,
      token_error::Signer { signer },
    }

    ensure! {
      self.domain == policy.domain.as_str(),
      token_error::Domain { domain: &self.domain },
    }

    let start = self.timestamp.saturating_sub(LEEWAY);

    ensure! {
      start <= policy.now,
      token_error::Pending { now: policy.now, start, timestamp: self.timestamp },
    }

    let expiry = self.timestamp.saturating_add(TTL).saturating_add(LEEWAY);

    ensure! {
      expiry >= policy.now,
      token_error::Expired { expiry, now: policy.now, timestamp: self.timestamp },
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const DOMAIN: &str = "foo";

  #[test]
  fn example() {
    let private_key = PrivateKey::zero();

    let claims = Claims {
      domain: "filepack.com".into(),
      timestamp: 0,
    };

    assert_eq!(private_key.sign(claims).to_string(), Token::EXAMPLE);
  }

  #[test]
  fn expired() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    Claims::verify(admin.public_key(), &DOMAIN.parse().unwrap(), 1090, &token).unwrap();
    assert_matches!(
      Claims::verify(admin.public_key(), &DOMAIN.parse().unwrap(), 1091, &token).unwrap_err(),
      TokenError::Expired {
        expiry: 1090,
        now: 1091,
        timestamp: 1000,
      },
    );
  }

  fn mint(private_key: &PrivateKey, timestamp: u64) -> String {
    private_key
      .sign(Claims {
        domain: DOMAIN.into(),
        timestamp,
      })
      .to_string()
  }

  #[test]
  fn pending() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    Claims::verify(admin.public_key(), &DOMAIN.parse().unwrap(), 970, &token).unwrap();
    assert_matches!(
      Claims::verify(admin.public_key(), &DOMAIN.parse().unwrap(), 969, &token).unwrap_err(),
      TokenError::Pending {
        now: 969,
        start: 970,
        timestamp: 1000,
      },
    );
  }

  #[test]
  fn tampered_claims() {
    let public_key = PublicKey::EXAMPLE.parse::<PublicKey>().unwrap();
    Claims::verify(
      public_key,
      &"filepack.com".parse().unwrap(),
      0,
      Token::EXAMPLE,
    )
    .unwrap();
    let token = Token::EXAMPLE.replacen("66696c657061636b2e636f6d", "66696c657061636b2e636f6e", 1);
    assert_matches!(
      Claims::verify(public_key, &"filepack.con".parse().unwrap(), 0, &token).unwrap_err(),
      TokenError::Signature {
        source: SignatureError::Invalid { public_key: signer, .. },
      } if signer == public_key,
    );
  }

  #[test]
  fn unknown_field_rejected() {
    /// Extra.
    #[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    struct Extra {
      /// Domain.
      #[n(1)]
      domain: String,
      /// Timestamp.
      #[n(2)]
      timestamp: u64,
      /// Unknown.
      #[n(3)]
      unknown: u64,
    }

    impl Message for Extra {
      const CONTEXT: Context = Context::Claims;
      const NAME: &'static str = "Extra";
      const SIGNATURE_EXAMPLE: &'static str = "";
      const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str = "";
      const SIGNATURE_TAG: Tag = Tag::Token;
      type Error = TokenError;
      type Policy<'a> = ();

      fn check(&self, _signer: PublicKey, _policy: ()) -> Result<(), TokenError> {
        Ok(())
      }
    }

    let admin = PrivateKey::generate();

    let token = admin
      .sign(Extra {
        domain: DOMAIN.into(),
        timestamp: 0,
        unknown: 0,
      })
      .to_string();

    assert_matches!(
      Claims::verify(admin.public_key(), &DOMAIN.parse().unwrap(), 0, &token).unwrap_err(),
      TokenError::Token {
        source: HexError::Decode {
          source: DecodeError::Unknown {
            source: UnknownError::Field { key: 3 },
            strict: true
          },
          tag: Tag::Token,
        },
      },
    );
  }

  #[test]
  fn wrong_domain() {
    let admin = PrivateKey::generate();
    let token = Claims::sign(&admin, "bar").unwrap();
    let error = Claims::verify(
      admin.public_key(),
      &DOMAIN.parse().unwrap(),
      now().unwrap(),
      &token,
    )
    .unwrap_err();
    assert_matches!(error, TokenError::Domain { ref domain } if domain == "bar");
    assert_eq!(error.to_string(), "token has incorrect domain `bar`");
  }

  #[test]
  fn wrong_signer() {
    let admin = PrivateKey::generate();
    let other = PrivateKey::generate();
    let token = mint(&other, 1000);
    assert_matches!(
      Claims::verify(admin.public_key(), &DOMAIN.parse().unwrap(), 0, &token).unwrap_err(),
      TokenError::Signer { signer } if signer == other.public_key(),
    );
  }
}
