use super::*;

const LEEWAY: u64 = 30;
const TTL: u64 = 60;

pub(crate) struct Policy<'a> {
  admin: PublicKey,
  domain: &'a Domain,
  method: &'a Method,
  now: u64,
  path: &'a str,
}

/// Claims made by an authorization token.
#[derive(Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Claims {
  /// Domain the token is valid for.
  #[n(1)]
  pub(crate) domain: String,
  /// HTTP method the token is valid for.
  #[n(2)]
  pub(crate) method: String,
  /// Request path and query the token is valid for.
  #[n(3)]
  pub(crate) path: String,
  /// Unix timestamp of signing in seconds.
  #[n(4)]
  pub(crate) timestamp: u64,
}

impl Claims {
  pub(crate) fn sign(
    private_key: &PrivateKey,
    domain: &str,
    method: &Method,
    path: &str,
  ) -> Result<String> {
    Ok(
      private_key
        .sign(Claims {
          domain: domain.into(),
          method: method.as_str().into(),
          path: path.into(),
          timestamp: now().context(error::Time)?,
        })
        .to_string(),
    )
  }

  pub(crate) fn verify(
    admin: PublicKey,
    domain: &Domain,
    method: &Method,
    now: u64,
    path: &str,
    token: &str,
  ) -> Result<(), TokenError> {
    let signature = token.parse::<Token>().context(token_error::Token)?;

    let policy = Policy {
      admin,
      domain,
      method,
      now,
      path,
    };

    signature.verify(policy)?;

    Ok(())
  }
}

impl Message for Claims {
  const CONTEXT: Context = Context::Claims;
  const NAME: &'static str = "Claims";
  const SIGNATURE_EXAMPLE: &'static str = concat!(
    "token101a03b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2902a4018c66696c6570",
    "61636b2e636f6d0284504f5354038c2f6170692f6d697373696e67040003c0bf5f315a3621c8ebc11acc022938e4",
    "9adcf788d8cccba9b52bd7d760e948a0330e91190ead5f0b2dd185a729f67c72ac3764ba0877ed7b63d7da39d08b",
    "1b5005",
  );
  const SIGNATURE_EXAMPLE_DESCRIPTION: &'static str = "Token with domain `filepack.com`, method \
    `POST`, path `/api/missing`, and timestamp 0, signed by the all-zero private key.";
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

    ensure! {
      self.method == policy.method.as_str(),
      token_error::Method { method: &self.method },
    }

    ensure! {
      self.path == policy.path,
      token_error::Path { path: &self.path },
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
  const METHOD: Method = Method::GET;
  const PATH: &str = "baz";

  #[test]
  fn example() {
    let private_key = PrivateKey::zero();

    let claims = Claims {
      domain: "filepack.com".into(),
      method: "POST".into(),
      path: "/api/missing".into(),
      timestamp: 0,
    };

    assert_eq!(private_key.sign(claims).to_string(), Token::EXAMPLE);
  }

  #[test]
  fn expired() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    verify(&admin, 1090, &token).unwrap();
    assert_matches!(
      verify(&admin, 1091, &token).unwrap_err(),
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
        method: METHOD.as_str().into(),
        path: PATH.into(),
        timestamp,
      })
      .to_string()
  }

  #[test]
  fn pending() {
    let admin = PrivateKey::generate();
    let token = mint(&admin, 1000);
    verify(&admin, 970, &token).unwrap();
    assert_matches!(
      verify(&admin, 969, &token).unwrap_err(),
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
      &Method::POST,
      0,
      "/api/missing",
      Token::EXAMPLE,
    )
    .unwrap();
    let token = Token::EXAMPLE.replacen("66696c657061636b2e636f6d", "66696c657061636b2e636f6e", 1);
    assert_matches!(
      Claims::verify(
        public_key,
        &"filepack.con".parse().unwrap(),
        &Method::POST,
        0,
        "/api/missing",
        &token,
      )
      .unwrap_err(),
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
      /// Method.
      #[n(2)]
      method: String,
      /// Path.
      #[n(3)]
      path: String,
      /// Timestamp.
      #[n(4)]
      timestamp: u64,
      /// Unknown.
      #[n(5)]
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
        method: METHOD.as_str().into(),
        path: PATH.into(),
        timestamp: 0,
        unknown: 0,
      })
      .to_string();

    assert_matches!(
      verify(&admin, 0, &token).unwrap_err(),
      TokenError::Token {
        source: HexError::Decode {
          source: DecodeError::Unknown {
            source: UnknownError::Field { key: 5 },
            strict: true
          },
          tag: Tag::Token,
        },
      },
    );
  }

  fn verify(admin: &PrivateKey, now: u64, token: &str) -> Result<(), TokenError> {
    Claims::verify(
      admin.public_key(),
      &DOMAIN.parse().unwrap(),
      &METHOD,
      now,
      PATH,
      token,
    )
  }

  #[test]
  fn wrong_domain() {
    let admin = PrivateKey::generate();
    let token = Claims::sign(&admin, "bar", &METHOD, PATH).unwrap();
    let error = verify(&admin, now().unwrap(), &token).unwrap_err();
    assert_matches!(error, TokenError::Domain { ref domain } if domain == "bar");
    assert_eq!(error.to_string(), "token has incorrect domain `bar`");
  }

  #[test]
  fn wrong_method() {
    let admin = PrivateKey::generate();
    let token = Claims::sign(&admin, DOMAIN, &Method::POST, PATH).unwrap();
    let error = verify(&admin, now().unwrap(), &token).unwrap_err();
    assert_matches!(error, TokenError::Method { ref method } if method == "POST");
    assert_eq!(error.to_string(), "token has incorrect method `POST`");
  }

  #[test]
  fn wrong_path() {
    let admin = PrivateKey::generate();
    let token = Claims::sign(&admin, DOMAIN, &METHOD, "baz?qux").unwrap();
    let error = verify(&admin, now().unwrap(), &token).unwrap_err();
    assert_matches!(error, TokenError::Path { ref path } if path == "baz?qux");
    assert_eq!(error.to_string(), "token has incorrect path `baz?qux`");
  }

  #[test]
  fn wrong_signer() {
    let admin = PrivateKey::generate();
    let other = PrivateKey::generate();
    let token = mint(&other, 1000);
    assert_matches!(
      verify(&admin, 0, &token).unwrap_err(),
      TokenError::Signer { signer } if signer == other.public_key(),
    );
  }
}
