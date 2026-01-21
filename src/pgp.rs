use {
  super::*,
  sequoia_openpgp::{
    Cert, Packet,
    cert::CertBuilder,
    crypto::mpi,
    packet::{
      self, Key,
      key::{Key6, SecretParts, UnspecifiedRole},
      signature::SignatureBuilder,
    },
    parse::Parse,
    policy::StandardPolicy,
    serialize::MarshalInto,
    types::{Curve, HashAlgorithm, SignatureType},
  },
  sha2::{Digest, Sha512},
  std::{os::unix::fs::PermissionsExt, process::Command},
};

#[test]
fn pgp_v4_signatures_can_be_verified() {
  let manifest = Manifest {
    files: Directory::new(),
    notes: Vec::new(),
  };

  let message = Message {
    fingerprint: manifest.fingerprint(),
    time: None,
  };

  let message = message.serialize();

  let policy = StandardPolicy::new();

  let (cert, _revocation) = CertBuilder::new()
    .add_userid("foo@bar")
    .add_signing_subkey()
    .generate()
    .unwrap();

  let signing_key = cert
    .keys()
    .unencrypted_secret()
    .with_policy(&policy, None)
    .supported()
    .alive()
    .revoked(false)
    .for_signing()
    .next()
    .unwrap();

  let mut keypair = signing_key.key().clone().into_keypair().unwrap();

  let signature_packet = SignatureBuilder::new(SignatureType::Binary)
    .set_hash_algo(HashAlgorithm::SHA512)
    .sign_message(&mut keypair, message.bytes())
    .unwrap();

  let mpi::Signature::EdDSA { r, s } = signature_packet.mpis() else {
    panic!("expected EdDSA signature");
  };

  let r_bytes = r.value_padded(32).unwrap();
  let s_bytes = s.value_padded(32).unwrap();

  let mut sig_bytes = [0u8; 64];
  sig_bytes[..32].copy_from_slice(&r_bytes);
  sig_bytes[32..].copy_from_slice(&s_bytes);

  let mpi::PublicKey::EdDSA { q, .. } = signing_key.key().mpis() else {
    panic!("expected EdDSA public key");
  };

  let (public_key_bytes, _) = q.decode_point(&Curve::Ed25519).unwrap();

  signature_packet
    .clone()
    .verify_message(signing_key.key(), message.bytes())
    .unwrap();

  {
    let hashed_area = signature_packet.hashed_area().to_vec().unwrap();
    let hashed_area_len = hashed_area.len();

    let mut header = [0u8; 6];
    header[0] = 4;
    header[1] = u8::from(signature_packet.typ());
    header[2] = u8::from(signature_packet.pk_algo());
    header[3] = u8::from(signature_packet.hash_algo());
    header[4..6].copy_from_slice(&(hashed_area_len as u16).to_be_bytes());

    let mut trailer = [0u8; 6];
    trailer[0] = 4;
    trailer[1] = 0xff;
    let len = (header.len() + hashed_area_len) as u32;
    trailer[2..6].copy_from_slice(&len.to_be_bytes());

    let mut hasher = Sha512::new();
    hasher.update(message.bytes());
    hasher.update(&header);
    hasher.update(&hashed_area);
    hasher.update(&trailer);
    let digest = hasher.finalize();

    let verifying_key =
      ed25519_dalek::VerifyingKey::from_bytes(public_key_bytes.try_into().unwrap()).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    verifying_key.verify_strict(&digest, &sig).unwrap();
  }

  let public_key = PublicKey::from_bytes(public_key_bytes.try_into().unwrap());

  let signature = Signature::new(
    SignatureScheme::Pgp {
      hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
    },
    ed25519_dalek::Signature::from_bytes(&sig_bytes),
  );

  signature.verify(&message, public_key).unwrap();
}

#[test]
#[ignore]
fn pgp_v6_signatures_can_be_verified() {
  let manifest = Manifest {
    files: Directory::new(),
    notes: Vec::new(),
  };

  let message = Message {
    fingerprint: manifest.fingerprint(),
    time: None,
  };

  let message = message.serialize();

  let secret_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());

  let key6: Key6<SecretParts, UnspecifiedRole> =
    Key6::import_secret_ed25519(secret_key.as_bytes(), None).unwrap();
  let key: Key<_, _> = key6.into();

  let mut keypair = key.clone().into_keypair().unwrap();

  let signature_packet = SignatureBuilder::new(SignatureType::Binary)
    .set_hash_algo(HashAlgorithm::SHA512)
    .sign_message(&mut keypair, message.bytes())
    .unwrap();

  assert_eq!(signature_packet.version(), 6);

  let packet::Signature::V6(sig6) = signature_packet.clone() else {
    panic!("expected v6 signature");
  };

  let salt = sig6.salt();

  let mpi::Signature::Ed25519 { s } = signature_packet.mpis() else {
    panic!("expected Ed25519 signature");
  };

  let mpi::PublicKey::Ed25519 { a } = key.mpis() else {
    panic!("expected Ed25519 public key");
  };

  signature_packet
    .clone()
    .verify_message(&key, message.bytes())
    .unwrap();

  {
    let hashed_area = signature_packet.hashed_area().to_vec().unwrap();
    let hashed_area_len = hashed_area.len();

    let mut header = [0u8; 8];
    header[0] = 6;
    header[1] = u8::from(signature_packet.typ());
    header[2] = u8::from(signature_packet.pk_algo());
    header[3] = u8::from(signature_packet.hash_algo());
    header[4..8].copy_from_slice(&(hashed_area_len as u32).to_be_bytes());

    let mut trailer = [0u8; 6];
    trailer[0] = 6;
    trailer[1] = 0xff;
    let len = (header.len() + hashed_area_len) as u32;
    trailer[2..6].copy_from_slice(&len.to_be_bytes());

    let mut hasher = Sha512::new();
    hasher.update(salt);
    hasher.update(message.bytes());
    hasher.update(&header);
    hasher.update(&hashed_area);
    hasher.update(&trailer);
    let digest = hasher.finalize();

    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(a).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(s.as_ref());
    verifying_key.verify_strict(&digest, &sig).unwrap();
  }

  let public_key = PublicKey::from_bytes(*a);

  let signature = Signature::new(
    SignatureScheme::Pgp {
      hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
    },
    ed25519_dalek::Signature::from_bytes(s.as_ref()),
  );

  signature.verify(&message, public_key).unwrap();
}

#[test]
fn gpg_v4_signatures_can_be_verified() {
  if cfg!(windows) {
    return;
  }

  let manifest = Manifest {
    files: Directory::new(),
    notes: Vec::new(),
  };

  let message = Message {
    fingerprint: manifest.fingerprint(),
    time: None,
  };

  let message = message.serialize();

  let tempdir = tempdir();

  let path = decode_path(tempdir.path()).unwrap();

  let gnupg_home = path.join("gnupg");
  fs::create_dir(&gnupg_home).unwrap();
  fs::set_permissions(&gnupg_home, Permissions::from_mode(0o700)).unwrap();

  let output = Command::new("gpg")
    .args(["--homedir", gnupg_home.as_str()])
    .arg("--batch")
    .args(["--passphrase", ""])
    .args(["--quick-gen-key", "foo@bar", "ed25519", "sign"])
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "gpg key generation failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let message_path = path.join("message");
  filesystem::write(&message_path, message.bytes()).unwrap();

  let signature_path = path.join("message.sig");
  let output = Command::new("gpg")
    .args(["--homedir", gnupg_home.as_str()])
    .arg("--batch")
    .args(["--passphrase", ""])
    .arg("--detach-sign")
    .args(["-o", signature_path.as_str()])
    .arg(message_path.as_str())
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "gpg signing failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let signature_bytes = fs::read(signature_path.as_std_path()).unwrap();
  let packet = Packet::from_bytes(&signature_bytes).unwrap();
  let Packet::Signature(signature_packet) = packet else {
    panic!("expected signature packet");
  };

  let mpi::Signature::EdDSA { r, s } = signature_packet.mpis() else {
    panic!("expected EdDSA signature");
  };

  let r_bytes = r.value_padded(32).unwrap();
  let s_bytes = s.value_padded(32).unwrap();

  let mut sig_bytes = [0u8; 64];
  sig_bytes[..32].copy_from_slice(&r_bytes);
  sig_bytes[32..].copy_from_slice(&s_bytes);

  let output = Command::new("gpg")
    .args(["--homedir", gnupg_home.as_str()])
    .args(["--export", "foo@bar"])
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "gpg export failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let cert = Cert::from_bytes(&output.stdout).unwrap();
  let policy = StandardPolicy::new();

  let signing_key = cert
    .keys()
    .with_policy(&policy, None)
    .supported()
    .for_signing()
    .next()
    .unwrap();

  let mpi::PublicKey::EdDSA { q, .. } = signing_key.key().mpis() else {
    panic!("expected EdDSA public key");
  };

  let (public_key_bytes, _) = q.decode_point(&Curve::Ed25519).unwrap();
  let public_key = PublicKey::from_bytes(public_key_bytes.try_into().unwrap());

  let output = Command::new("gpg")
    .args(["--homedir", gnupg_home.as_str()])
    .arg("--batch")
    .args(["--passphrase", ""])
    .args(["--export-secret-keys", "foo@bar"])
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "gpg export-secret-keys failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let secret_cert = Cert::from_bytes(&output.stdout).unwrap();

  let secret_key = secret_cert
    .keys()
    .unencrypted_secret()
    .with_policy(&policy, None)
    .supported()
    .for_signing()
    .next()
    .unwrap();

  let packet::key::SecretKeyMaterial::Unencrypted(secret_key_material) =
    secret_key.key().optional_secret().unwrap()
  else {
    panic!("expected unencrypted secret key");
  };

  let mpi::SecretKeyMaterial::EdDSA { scalar } = secret_key_material.map(|m| m.clone()) else {
    panic!("expected EdDSA secret key");
  };

  let private_key = PrivateKey::from_bytes(scalar.value().try_into().unwrap());

  assert_eq!(private_key.public_key(), public_key);

  let signature = Signature::new(
    SignatureScheme::Pgp {
      hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
    },
    ed25519_dalek::Signature::from_bytes(&sig_bytes),
  );

  signature.verify(&message, public_key).unwrap();
}
