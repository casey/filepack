use {
  super::*,
  axum::http::{HeaderMap, header},
  jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation},
};

const LEEWAY: u64 = 30;

const PKCS8_PREFIX: [u8; 16] = [
  0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
];

const TTL: u64 = 60;

pub(crate) struct AuthConfig {
  pub(crate) admin: PublicKey,
  pub(crate) audiences: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Claims {
  aud: String,
  exp: u64,
  iat: u64,
  iss: String,
  nbf: u64,
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

  let der = pkcs8(&private_key.as_secret_bytes());

  jsonwebtoken::encode(
    &Header::new(Algorithm::EdDSA),
    &claims,
    &EncodingKey::from_ed_der(&der),
  )
  .context(error::JwtEncode)
}

fn pkcs8(seed: &[u8; PrivateKey::LEN]) -> [u8; 48] {
  let mut der = [0; 48];
  der[..16].copy_from_slice(&PKCS8_PREFIX);
  der[16..].copy_from_slice(seed);
  der
}

pub(crate) fn verify(auth: &AuthConfig, headers: &HeaderMap) -> ServerResult {
  let token = headers
    .get(header::AUTHORIZATION)
    .context(server_error::UploadAuthMissing)?
    .to_str()
    .ok()
    .and_then(|value| value.strip_prefix("Bearer "))
    .context(server_error::UploadAuthMalformed)?;

  let key = DecodingKey::from_ed_der(auth.admin.inner().as_bytes());

  let mut validation = Validation::new(Algorithm::EdDSA);
  validation.leeway = LEEWAY;
  validation.validate_nbf = true;
  validation.set_required_spec_claims(&["aud", "exp", "iss", "nbf"]);
  validation.set_audience(&auth.audiences);
  validation.set_issuer(&[auth.admin.to_string()]);

  jsonwebtoken::decode::<Claims>(token, &key, &validation)
    .context(server_error::UploadAuthInvalid)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use {super::*, axum::http::HeaderValue, jsonwebtoken::errors::ErrorKind};

  fn config(admin: PublicKey) -> AuthConfig {
    AuthConfig {
      admin,
      audiences: vec!["filepack.example".into()],
    }
  }

  #[test]
  fn expired() {
    let private_key = PrivateKey::generate();
    let auth = config(private_key.public_key());
    let iat = now().unwrap() - LEEWAY - TTL - 10;
    let token = mint(
      &private_key,
      &Claims {
        aud: "filepack.example".into(),
        exp: iat + TTL,
        iat,
        iss: private_key.public_key().to_string(),
        nbf: iat,
      },
    );
    assert_matches!(
      verify(&auth, &headers(&token)).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::ExpiredSignature),
    );
  }

  fn headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
      header::AUTHORIZATION,
      HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    headers
  }

  #[test]
  fn malformed_header() {
    let private_key = PrivateKey::generate();
    let auth = config(private_key.public_key());
    let mut headers = HeaderMap::new();
    headers.insert(
      header::AUTHORIZATION,
      HeaderValue::from_static("not bearer"),
    );
    assert_matches!(
      verify(&auth, &headers).unwrap_err(),
      ServerError::UploadAuthMalformed,
    );
  }

  fn mint(private_key: &PrivateKey, claims: &Claims) -> String {
    let der = pkcs8(&private_key.as_secret_bytes());
    jsonwebtoken::encode(
      &Header::new(Algorithm::EdDSA),
      claims,
      &EncodingKey::from_ed_der(&der),
    )
    .unwrap()
  }

  #[test]
  fn missing_header() {
    let private_key = PrivateKey::generate();
    let auth = config(private_key.public_key());
    assert_matches!(
      verify(&auth, &HeaderMap::new()).unwrap_err(),
      ServerError::UploadAuthMissing,
    );
  }

  #[test]
  fn pkcs8_prefix_is_correct_length() {
    assert_eq!(PKCS8_PREFIX.len(), 16);
  }

  #[test]
  fn roundtrip() {
    let private_key = PrivateKey::generate();
    let auth = config(private_key.public_key());
    let token = encode(&private_key, "filepack.example").unwrap();
    verify(&auth, &headers(&token)).unwrap();
  }

  #[test]
  fn wrong_audience() {
    let private_key = PrivateKey::generate();
    let auth = config(private_key.public_key());
    let token = encode(&private_key, "evil.example").unwrap();
    assert_matches!(
      verify(&auth, &headers(&token)).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::InvalidAudience),
    );
  }

  #[test]
  fn wrong_issuer() {
    let admin = PrivateKey::generate();
    let auth = config(admin.public_key());
    let iat = now().unwrap();
    let token = mint(
      &admin,
      &Claims {
        aud: "filepack.example".into(),
        exp: iat + TTL,
        iat,
        iss: PrivateKey::generate().public_key().to_string(),
        nbf: iat,
      },
    );
    assert_matches!(
      verify(&auth, &headers(&token)).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::InvalidIssuer),
    );
  }

  #[test]
  fn wrong_signer() {
    let admin = PrivateKey::generate();
    let intruder = PrivateKey::generate();
    let auth = config(admin.public_key());
    let token = encode(&intruder, "filepack.example").unwrap();
    assert_matches!(
      verify(&auth, &headers(&token)).unwrap_err(),
      ServerError::UploadAuthInvalid { source } if matches!(source.kind(), ErrorKind::InvalidSignature),
    );
  }
}
