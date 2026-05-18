use {super::*, ed25519_dalek::pkcs8::EncodePrivateKey, rustls::pki_types::PrivatePkcs8KeyDer};

pub fn generate_server_certificate(subject: &str) -> (String, String) {
  let key_pair = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).unwrap();

  let cert = rcgen::CertificateParams::new(vec![subject.into()])
    .unwrap()
    .self_signed(&key_pair)
    .unwrap();

  (cert.pem(), key_pair.serialize_pem())
}

pub fn self_signed_cert(private_key: &PrivateKey) -> (String, String) {
  let pkcs8 = private_key.inner_secret().to_pkcs8_der().unwrap();

  let key_pair = rcgen::KeyPair::from_pkcs8_der_and_sign_algo(
    &PrivatePkcs8KeyDer::from(pkcs8.as_bytes()),
    &rcgen::PKCS_ED25519,
  )
  .unwrap();

  let cert = rcgen::CertificateParams::new(vec!["filepack".into()])
    .unwrap()
    .self_signed(&key_pair)
    .unwrap();

  (cert.pem(), key_pair.serialize_pem())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let private_key = PrivateKey::generate();
    let (cert_pem, key_pem) = self_signed_cert(&private_key);
    assert!(cert_pem.starts_with("-----BEGIN CERTIFICATE-----"));
    assert!(key_pem.starts_with("-----BEGIN PRIVATE KEY-----"));
  }

  #[test]
  fn server_cert_round_trip() {
    let (cert_pem, key_pem) = generate_server_certificate("foo");
    assert!(cert_pem.starts_with("-----BEGIN CERTIFICATE-----"));
    assert!(key_pem.starts_with("-----BEGIN PRIVATE KEY-----"));
  }
}
