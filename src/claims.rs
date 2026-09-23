use super::*;

const LEEWAY: u64 = 30;
const TTL: u64 = 60;

pub(crate) struct Policy<'a> {
  admin: PublicKey,
  audience: Option<&'a str>,
  now: u64,
}

#[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
#[deco(strict)]
pub(crate) struct Claims {
  #[n(0)]
  pub(crate) audience: String,
  #[n(1)]
  pub(crate) timestamp: u64,
}

impl Claims {
  pub(crate) fn sign(private_key: &PrivateKey, audience: &str) -> Result<String> {
    Ok(
      private_key
        .sign(Claims {
          audience: audience.into(),
          timestamp: now().context(error::Time)?,
        })
        .to_string(),
    )
  }

  pub(crate) fn verify(
    admin: PublicKey,
    audience: Option<&str>,
    now: u64,
    token: &str,
  ) -> Result<(), AuthorizationError> {
    let signature = token.parse::<Token>().context(authorization_error::Token)?;

    let policy = Policy {
      admin,
      audience,
      now,
    };

    signature.verify(policy)?;

    Ok(())
  }
}

impl Message for Claims {
  const CONTEXT: Context = Context::Claims;
  const TAG: Tag = Tag::Token;
  type Error = AuthorizationError;
  type Policy<'a> = Policy<'a>;

  fn check(&self, signer: PublicKey, policy: Policy) -> Result<(), AuthorizationError> {
    ensure! {
      signer == policy.admin,
      authorization_error::Signer { signer },
    }

    ensure! {
      policy.audience.is_some_and(|audience| self.audience == audience),
      authorization_error::Audience { audience: &self.audience },
    }

    let start = self.timestamp.saturating_sub(LEEWAY);

    ensure! {
      start <= policy.now,
      authorization_error::Pending { now: policy.now, start, timestamp: self.timestamp },
    }

    let expiry = self.timestamp.saturating_add(TTL).saturating_add(LEEWAY);

    ensure! {
      expiry >= policy.now,
      authorization_error::Expired { expiry, now: policy.now, timestamp: self.timestamp },
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const AUDIENCE: &str = "foo";

  #[test]
  fn expired() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    Claims::verify(admin.public_key(), Some(AUDIENCE), 1090, &token).unwrap();
    assert_matches!(
      Claims::verify(admin.public_key(), Some(AUDIENCE), 1091, &token).unwrap_err(),
      AuthorizationError::Expired {
        expiry: 1090,
        now: 1091,
        timestamp: 1000,
      },
    );
  }

  fn mint(private_key: &PrivateKey, timestamp: u64) -> String {
    private_key
      .sign(Claims {
        audience: AUDIENCE.into(),
        timestamp,
      })
      .to_string()
  }

  #[test]
  fn missing_audience() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    let error = Claims::verify(admin.public_key(), None, 1000, &token).unwrap_err();
    assert_matches!(error, AuthorizationError::Audience { ref audience } if audience == AUDIENCE);
  }

  #[test]
  fn pending() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    Claims::verify(admin.public_key(), Some(AUDIENCE), 970, &token).unwrap();
    assert_matches!(
      Claims::verify(admin.public_key(), Some(AUDIENCE), 969, &token).unwrap_err(),
      AuthorizationError::Pending {
        now: 969,
        start: 970,
        timestamp: 1000,
      },
    );
  }

  #[test]
  fn tampered_claims() {
    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();
    Claims::verify(public_key, Some(AUDIENCE), 0, test::TOKEN).unwrap();
    let token = test::TOKEN.replacen("666f6f", "666f70", 1);
    assert_matches!(
      Claims::verify(public_key, Some("fop"), 0, &token).unwrap_err(),
      AuthorizationError::Signature {
        source: SignatureError::Invalid { public_key: signer, .. },
      } if signer == public_key,
    );
  }

  #[test]
  fn unknown_field_rejected() {
    #[derive(Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    struct Extra {
      #[n(0)]
      audience: String,
      #[n(1)]
      timestamp: u64,
      #[n(2)]
      unknown: u64,
    }

    impl Message for Extra {
      const CONTEXT: Context = Context::Claims;
      const TAG: Tag = Tag::Token;
      type Error = AuthorizationError;
      type Policy<'a> = ();

      fn check(&self, _signer: PublicKey, _policy: ()) -> Result<(), AuthorizationError> {
        Ok(())
      }
    }

    let admin = PrivateKey::generate();

    let token = admin
      .sign(Extra {
        audience: AUDIENCE.into(),
        timestamp: 0,
        unknown: 0,
      })
      .to_string();

    assert_matches!(
      Claims::verify(admin.public_key(), Some(AUDIENCE), 0, &token).unwrap_err(),
      AuthorizationError::Token {
        source: HexError::Decode {
          source: DecodeError::UnknownField { key: 2 },
          tag: Tag::Token,
        },
      },
    );
  }

  #[test]
  fn wrong_audience() {
    let admin = PrivateKey::generate();
    let token = Claims::sign(&admin, "bar").unwrap();
    let error =
      Claims::verify(admin.public_key(), Some(AUDIENCE), now().unwrap(), &token).unwrap_err();
    assert_matches!(error, AuthorizationError::Audience { ref audience } if audience == "bar");
    assert_eq!(error.to_string(), "token has incorrect audience `bar`");
  }

  #[test]
  fn wrong_signer() {
    let admin = PrivateKey::generate();
    let other = PrivateKey::generate();
    let token = mint(&other, 1000);
    assert_matches!(
      Claims::verify(admin.public_key(), Some(AUDIENCE), 0, &token).unwrap_err(),
      AuthorizationError::Signer { signer } if signer == other.public_key(),
    );
  }
}
