use super::*;

#[test]
fn ssh_keygen_signatures_can_be_verified() {
  let message_bytes = include_bytes!("../static/ssh-test/message");
  let signature_str = include_str!("../static/ssh-test/message.sig");
  let public_key_str = include_str!("../static/ssh-test/id_ed25519.pub");
  let private_key_str = include_str!("../static/ssh-test/id_ed25519");

  let message = SerializedMessage(message_bytes.to_vec());

  let public_key = {
    let public_key = public_key_str.trim().parse::<ssh_key::PublicKey>().unwrap();

    let ssh_key::public::KeyData::Ed25519(public_key) = public_key.key_data() else {
      panic!("expected ed25519");
    };

    PublicKey::from_bytes(public_key.0).unwrap()
  };

  let private_key = {
    let private_key = ssh_key::PrivateKey::from_openssh(private_key_str).unwrap();

    let ssh_key::private::KeypairData::Ed25519(keypair) = private_key.key_data() else {
      panic!("expected ed25519");
    };

    assert_eq!(
      PublicKey::from_bytes(*keypair.public.as_ref()).unwrap(),
      public_key,
    );

    let private_key = PrivateKey::from_bytes(keypair.private.to_bytes());

    assert_eq!(private_key.public_key(), public_key);

    private_key
  };

  let signature = {
    let signature = signature_str.parse::<ssh_key::SshSig>().unwrap();
    Signature::new(
      SignatureScheme::Ssh,
      ed25519_dalek::Signature::from_bytes(&signature.signature_bytes().try_into().unwrap()),
    )
  };

  signature.verify(&message, public_key).unwrap();

  let public_key = public_key.to_string();
  let signature = signature.to_string();

  Test::new()
    .write(
      "filepack.json",
      json! {
        files: {},
        notes: [{
          signatures: {
            *public_key: signature,
          },
        }],
      },
    )
    .args(["verify", "--key", &public_key])
    .stderr("successfully verified 0 files with 1 signature across 1 note\n")
    .success();
}

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

  let public_key = public_key.to_string();

  let signature = signature.to_string();

  Test::new()
    .write(
      "filepack.json",
      json! {
        files: {},
        notes: [{
          signatures: {
            *public_key: signature,
          },
        }],
      },
    )
    .args(["verify", "--key", &public_key])
    .stderr("successfully verified 0 files with 1 signature across 1 note\n")
    .success();
}
