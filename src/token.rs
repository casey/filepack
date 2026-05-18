use {
  super::*,
  ed25519_dalek::pkcs8::EncodePrivateKey,
  jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation},
};

const LEEWAY: u64 = 30;

const TTL: u64 = 60;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Token {
  #[serde(rename = "aud")]
  audience: String,
  #[serde(rename = "exp")]
  expiration_time: u64,
  #[serde(rename = "iat")]
  issued_at_time: u64,
  #[serde(rename = "iss")]
  issuer: String,
  #[serde(rename = "nbf")]
  not_before_time: u64,
}

impl Token {
  pub(crate) fn encode(private_key: &PrivateKey, audience: &str) -> Result<String> {
    let now = now()?;

    let claims = Self {
      audience: audience.into(),
      expiration_time: now + TTL,
      issued_at_time: now,
      issuer: private_key.public_key().to_string(),
      not_before_time: now,
    };

    let der = private_key.inner_secret().to_pkcs8_der().unwrap();

    Ok(
      jsonwebtoken::encode(
        &Header::new(Algorithm::EdDSA),
        &claims,
        &EncodingKey::from_ed_der(der.as_bytes()),
      )
      .unwrap(),
    )
  }

  pub(crate) fn verify(admin: PublicKey, audiences: &[String], token: &str) -> ServerResult {
    let key = DecodingKey::from_ed_der(admin.inner().as_bytes());

    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = LEEWAY;
    validation.validate_nbf = true;
    validation.set_audience(audiences);
    validation.set_issuer(&[admin.to_string()]);

    jsonwebtoken::decode::<Self>(token, &key, &validation)
      .context(server_error::AuthorizationInvalid)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, jsonwebtoken::errors::ErrorKind};

  const AUDIENCE: &str = "filepack.example";

  fn audiences() -> Vec<String> {
    vec![AUDIENCE.into()]
  }

  #[test]
  fn expired() {
    let private_key = PrivateKey::generate();
    let now = now().unwrap() - LEEWAY - TTL - 10;
    let token = mint(
      &private_key,
      &Token {
        audience: AUDIENCE.into(),
        expiration_time: now + TTL,
        issued_at_time: now,
        issuer: private_key.public_key().to_string(),
        not_before_time: now,
      },
    );
    assert_matches!(
      Token::verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::AuthorizationInvalid { source } if matches!(source.kind(), ErrorKind::ExpiredSignature),
    );
  }

  #[test]
  fn future_nbf() {
    let private_key = PrivateKey::generate();
    let now = now().unwrap();
    let token = mint(
      &private_key,
      &Token {
        audience: AUDIENCE.into(),
        expiration_time: now + TTL,
        issued_at_time: now,
        issuer: private_key.public_key().to_string(),
        not_before_time: now + LEEWAY + 10,
      },
    );
    assert_matches!(
      Token::verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::AuthorizationInvalid { source } if matches!(source.kind(), ErrorKind::ImmatureSignature),
    );
  }

  fn mint(private_key: &PrivateKey, claims: &Token) -> String {
    let der = private_key.inner_secret().to_pkcs8_der().unwrap();
    jsonwebtoken::encode(
      &Header::new(Algorithm::EdDSA),
      claims,
      &EncodingKey::from_ed_der(der.as_bytes()),
    )
    .unwrap()
  }

  #[test]
  fn roundtrip() {
    let private_key = PrivateKey::generate();
    let token = Token::encode(&private_key, AUDIENCE).unwrap();
    Token::verify(private_key.public_key(), &audiences(), &token).unwrap();
  }

  #[test]
  fn unknown_claim_rejected() {
    #[derive(Serialize)]
    struct ExtraClaims {
      aud: String,
      exp: u64,
      extra: String,
      iat: u64,
      iss: String,
      nbf: u64,
    }

    let private_key = PrivateKey::generate();
    let iat = now().unwrap();
    let claims = ExtraClaims {
      aud: AUDIENCE.into(),
      exp: iat + TTL,
      extra: "junk".into(),
      iat,
      iss: private_key.public_key().to_string(),
      nbf: iat,
    };
    let der = private_key.inner_secret().to_pkcs8_der().unwrap();
    let token = jsonwebtoken::encode(
      &Header::new(Algorithm::EdDSA),
      &claims,
      &EncodingKey::from_ed_der(der.as_bytes()),
    )
    .unwrap();
    assert_matches!(
      Token::verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::AuthorizationInvalid { source } if matches!(source.kind(), ErrorKind::Json(_)),
    );
  }

  #[test]
  fn wrong_audience() {
    let private_key = PrivateKey::generate();
    let token = Token::encode(&private_key, "evil.example").unwrap();
    assert_matches!(
      Token::verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::AuthorizationInvalid { source } if matches!(source.kind(), ErrorKind::InvalidAudience),
    );
  }

  #[test]
  fn wrong_issuer() {
    let admin = PrivateKey::generate();
    let now = now().unwrap();
    let token = mint(
      &admin,
      &Token {
        audience: AUDIENCE.into(),
        expiration_time: now + TTL,
        issued_at_time: now,
        issuer: PrivateKey::generate().public_key().to_string(),
        not_before_time: now,
      },
    );
    assert_matches!(
      Token::verify(admin.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::AuthorizationInvalid { source } if matches!(source.kind(), ErrorKind::InvalidIssuer),
    );
  }

  #[test]
  fn wrong_signer() {
    let admin = PrivateKey::generate();
    let intruder = PrivateKey::generate();
    let token = Token::encode(&intruder, AUDIENCE).unwrap();
    assert_matches!(
      Token::verify(admin.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::AuthorizationInvalid { source } if matches!(source.kind(), ErrorKind::InvalidSignature),
    );
  }
}
