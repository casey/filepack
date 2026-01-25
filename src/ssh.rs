use super::*;

#[test]
fn ssh_signatures_can_be_generated_and_verified() {
  use {
    rand::rngs::OsRng,
    ssh_key::{Algorithm, HashAlg},
  };

  let message = {
    let manifest = Manifest {
      files: Directory::new(),
      notes: Vec::new(),
    };

    Message {
      fingerprint: manifest.fingerprint(),
      time: None,
    }
    .serialize()
  };

  let private_key = ssh_key::PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();

  let ssh_sig = private_key
    .sign("filepack", HashAlg::Sha512, message.as_bytes())
    .unwrap();

  let signature = {
    let sig_bytes: [u8; 64] = ssh_sig.signature_bytes().try_into().unwrap();
    Signature::new(
      SignatureScheme::Ssh,
      ed25519_dalek::Signature::from_bytes(&sig_bytes),
    )
  };

  let public_key = {
    let ssh_key::public::KeyData::Ed25519(ed25519_key) = private_key.public_key().key_data() else {
      panic!("expected ed25519");
    };

    PublicKey::from_bytes(ed25519_key.0).unwrap()
  };

  signature.verify(&message, public_key).unwrap();
}
