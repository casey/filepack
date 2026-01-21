use {
  super::*,
  sequoia_openpgp::{
    self as openpgp,
    cert::prelude::*,
    crypto::mpi,
    packet::{
      key::{Key6, SecretParts, UnspecifiedRole},
      signature::SignatureBuilder,
    },
    policy::StandardPolicy,
    serialize::MarshalInto,
    types::{HashAlgorithm, SignatureType},
  },
  sha2::{Digest, Sha512},
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
    .sign_message(&mut keypair, message.filepack_signed_data())
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

  let (public_key_bytes, _) = q.decode_point(&openpgp::types::Curve::Ed25519).unwrap();

  signature_packet
    .clone()
    .verify_message(signing_key.key(), message.filepack_signed_data())
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
    hasher.update(message.filepack_signed_data());
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
  let key: openpgp::packet::Key<_, _> = key6.into();

  let mut keypair = key.clone().into_keypair().unwrap();

  let signature_packet = SignatureBuilder::new(SignatureType::Binary)
    .set_hash_algo(HashAlgorithm::SHA512)
    .sign_message(&mut keypair, message.filepack_signed_data())
    .unwrap();

  assert_eq!(signature_packet.version(), 6);

  let openpgp::packet::Signature::V6(sig6) = signature_packet.clone() else {
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
    .verify_message(&key, message.filepack_signed_data())
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
    hasher.update(message.filepack_signed_data());
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
