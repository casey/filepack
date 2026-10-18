use super::*;

const LEEWAY: u64 = 30;
const TTL: u64 = 60;

pub(crate) struct Policy<'a> {
  admin: PublicKey,
  audience: Option<&'a str>,
  now: u64,
}

/// Claims made by an authorization token.
#[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Claims {
  /// Domain the token is valid for.
  #[n(1)]
  pub(crate) audience: String,
  /// Unix timestamp of signing in seconds.
  #[n(2)]
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
  const EXAMPLE: &'static str = concat!(
    "token101a03b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da290290018c66696c6570",
    "61636b2e636f6d020003c086fe820f47ef0c349617059f30b3adf4a13ffd6a3217782dfeabbe99fc905742dfc12b",
    "ea0ab7dca5dfc05580041ce942c3417dd1a28d5640a9de8cf9daca640f",
  );
  const NAME: &'static str = "Claims";
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
  fn example() {
    let private_key = PrivateKey::from([0; PrivateKey::LEN]);

    let claims = Claims {
      audience: "filepack.com".into(),
      timestamp: 0,
    };

    assert_eq!(private_key.sign(claims).to_string(), Token::EXAMPLE);
  }

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
    let public_key = PublicKey::EXAMPLE.parse::<PublicKey>().unwrap();
    Claims::verify(public_key, Some("filepack.com"), 0, Token::EXAMPLE).unwrap();
    let token = Token::EXAMPLE.replacen("66696c657061636b2e636f6d", "66696c657061636b2e636f6e", 1);
    assert_matches!(
      Claims::verify(public_key, Some("filepack.con"), 0, &token).unwrap_err(),
      AuthorizationError::Signature {
        source: SignatureError::Invalid { public_key: signer, .. },
      } if signer == public_key,
    );
  }

  #[test]
  fn unknown_field_rejected() {
    /// Extra.
    #[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
    struct Extra {
      /// Audience.
      #[n(1)]
      audience: String,
      /// Timestamp.
      #[n(2)]
      timestamp: u64,
      /// Unknown.
      #[n(3)]
      unknown: u64,
    }

    impl Message for Extra {
      const CONTEXT: Context = Context::Claims;
      const EXAMPLE: &'static str = "";
      const NAME: &'static str = "Extra";
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
