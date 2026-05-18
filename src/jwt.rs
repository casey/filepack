use {
  super::*,
  axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts},
  axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
  },
  ed25519_dalek::pkcs8::EncodePrivateKey,
  jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation},
};

const LEEWAY: u64 = 30;

const TTL: u64 = 60;

pub(crate) struct AuthConfig {
  pub(crate) admin: Option<PublicKey>,
  pub(crate) audiences: Vec<String>,
}

pub(crate) struct Authenticated;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claims {
  aud: String,
  exp: u64,
  iat: u64,
  iss: String,
  nbf: u64,
}

impl<S: Send + Sync> FromRequestParts<S> for Authenticated {
  type Rejection = ServerError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> ServerResult<Self> {
    let Some(auth) = parts.extensions.get::<Arc<AuthConfig>>().cloned() else {
      return Ok(Self);
    };

    let admin = auth.admin.context(server_error::UploadForbidden)?;

    let TypedHeader(Authorization(bearer)) = parts
      .extract::<TypedHeader<Authorization<Bearer>>>()
      .await
      .map_err(|err| {
        if err.is_missing() {
          server_error::UploadAuthMissing.build()
        } else {
          server_error::UploadAuthMalformed.build()
        }
      })?;

    verify(admin, &auth.audiences, bearer.token())?;

    Ok(Self)
  }
}

pub(crate) fn encode(private_key: &PrivateKey, audience: &str) -> Result<String> {
  let iat = now()?;

  let claims = Claims {
    aud: audience.into(),
    exp: iat + TTL,
    iat,
    iss: private_key.public_key().to_string(),
    nbf: iat,
  };

  let der = private_key.inner_secret().to_pkcs8_der().unwrap();

  jsonwebtoken::encode(
    &Header::new(Algorithm::EdDSA),
    &claims,
    &EncodingKey::from_ed_der(der.as_bytes()),
  )
  .context(error::JwtEncode)
}

pub(crate) fn verify(admin: PublicKey, audiences: &[String], token: &str) -> ServerResult {
  let key = DecodingKey::from_ed_der(admin.inner().as_bytes());

  let mut validation = Validation::new(Algorithm::EdDSA);
  validation.leeway = LEEWAY;
  validation.validate_nbf = true;
  validation.set_audience(audiences);
  validation.set_issuer(&[admin.to_string()]);

  jsonwebtoken::decode::<Claims>(token, &key, &validation)
    .context(server_error::UploadAuthInvalid)?;

  Ok(())
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
    let iat = now().unwrap() - LEEWAY - TTL - 10;
    let token = mint(
      &private_key,
      &Claims {
        aud: AUDIENCE.into(),
        exp: iat + TTL,
        iat,
        iss: private_key.public_key().to_string(),
        nbf: iat,
      },
    );
    assert_matches!(
      verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::ExpiredSignature),
    );
  }

  #[test]
  fn future_nbf() {
    let private_key = PrivateKey::generate();
    let iat = now().unwrap();
    let token = mint(
      &private_key,
      &Claims {
        aud: AUDIENCE.into(),
        exp: iat + TTL,
        iat,
        iss: private_key.public_key().to_string(),
        nbf: iat + LEEWAY + 10,
      },
    );
    assert_matches!(
      verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::ImmatureSignature),
    );
  }

  fn mint(private_key: &PrivateKey, claims: &Claims) -> String {
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
    let token = encode(&private_key, AUDIENCE).unwrap();
    verify(private_key.public_key(), &audiences(), &token).unwrap();
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
      verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::Json(_)),
    );
  }

  #[test]
  fn wrong_audience() {
    let private_key = PrivateKey::generate();
    let token = encode(&private_key, "evil.example").unwrap();
    assert_matches!(
      verify(private_key.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::InvalidAudience),
    );
  }

  #[test]
  fn wrong_issuer() {
    let admin = PrivateKey::generate();
    let iat = now().unwrap();
    let token = mint(
      &admin,
      &Claims {
        aud: AUDIENCE.into(),
        exp: iat + TTL,
        iat,
        iss: PrivateKey::generate().public_key().to_string(),
        nbf: iat,
      },
    );
    assert_matches!(
      verify(admin.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::InvalidIssuer),
    );
  }

  #[test]
  fn wrong_signer() {
    let admin = PrivateKey::generate();
    let intruder = PrivateKey::generate();
    let token = encode(&intruder, AUDIENCE).unwrap();
    assert_matches!(
      verify(admin.public_key(), &audiences(), &token).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::InvalidSignature),
    );
  }
}
