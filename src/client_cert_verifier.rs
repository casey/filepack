use {
  super::*,
  rustls::{
    DigitallySignedStruct, DistinguishedName, SignatureScheme,
    client::danger::HandshakeSignatureValid,
    crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, UnixTime},
    server::danger::{ClientCertVerified, ClientCertVerifier},
  },
};

const ED25519_OID: &str = "1.3.101.112";

#[derive(Debug)]
pub(crate) struct Verifier {
  pub(crate) allowed: PublicKey,
  pub(crate) provider: Arc<CryptoProvider>,
}

impl ClientCertVerifier for Verifier {
  fn client_auth_mandatory(&self) -> bool {
    false
  }

  fn offer_client_auth(&self) -> bool {
    true
  }

  fn root_hint_subjects(&self) -> &[DistinguishedName] {
    &[]
  }

  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
    self
      .provider
      .signature_verification_algorithms
      .supported_schemes()
  }

  fn verify_client_cert(
    &self,
    end_entity: &CertificateDer<'_>,
    _intermediates: &[CertificateDer<'_>],
    _now: UnixTime,
  ) -> Result<ClientCertVerified, rustls::Error> {
    let public_key = extract_ed25519_pubkey(end_entity);
    assert_eq!(public_key, self.allowed);
    Ok(ClientCertVerified::assertion())
  }

  fn verify_tls12_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    verify_tls12_signature(
      message,
      cert,
      dss,
      &self.provider.signature_verification_algorithms,
    )
  }

  fn verify_tls13_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    verify_tls13_signature(
      message,
      cert,
      dss,
      &self.provider.signature_verification_algorithms,
    )
  }
}

pub(crate) fn extract_ed25519_pubkey(cert: &CertificateDer<'_>) -> PublicKey {
  let (_, parsed) = x509_parser::parse_x509_certificate(cert.as_ref()).unwrap();

  let spki = parsed.public_key();

  assert_eq!(spki.algorithm.algorithm.to_id_string(), ED25519_OID);

  let bytes: [u8; PublicKey::LEN] = spki.subject_public_key.data.as_ref().try_into().unwrap();

  PublicKey::from_bytes(bytes).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extract_matches_input() {
    let private_key = PrivateKey::generate();
    let (cert_pem, _) = crate::tls::self_signed_cert(&private_key);
    let der = rustls::pki_types::pem::PemObject::from_pem_slice(cert_pem.as_bytes()).unwrap();
    assert_eq!(extract_ed25519_pubkey(&der), private_key.public_key());
  }
}
