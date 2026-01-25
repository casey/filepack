use {
  super::*,
  sequoia_openpgp::{
    Cert, Packet,
    cert::CertBuilder,
    crypto::mpi,
    packet::{self, signature::SignatureBuilder},
    parse::Parse,
    policy::StandardPolicy,
    serialize::MarshalInto,
    types::{Curve, HashAlgorithm, SignatureType},
  },
  sha2::{Digest, Sha512},
};

#[test]
fn gpg_v4_signatures_can_be_verified() {
  let policy = StandardPolicy::new();

  let message_bytes = include_bytes!("../static/gpg-test/message");
  let signature_bytes = include_bytes!("../static/gpg-test/message.sig");
  let public_key_bytes = include_bytes!("../static/gpg-test/public-key.gpg");
  let secret_key_bytes = include_bytes!("../static/gpg-test/secret-key.gpg");

  let message = SerializedMessage(message_bytes.to_vec());

  let public_key = {
    let cert = Cert::from_bytes(public_key_bytes).unwrap();

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
    PublicKey::from_bytes(public_key_bytes.try_into().unwrap()).unwrap()
  };

  {
    let secret_cert = Cert::from_bytes(secret_key_bytes).unwrap();

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

    let mpi::SecretKeyMaterial::EdDSA { scalar } = secret_key_material.map(Clone::clone) else {
      panic!("expected EdDSA secret key");
    };

    let private_key = PrivateKey::from_bytes(scalar.value().try_into().unwrap());

    assert_eq!(private_key.public_key(), public_key);
  }

  let signature = {
    let packet = Packet::from_bytes(signature_bytes).unwrap();
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

    Signature::new(
      SignatureScheme::Pgp {
        hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
      },
      ed25519_dalek::Signature::from_bytes(&sig_bytes),
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
fn pgp_v4_signatures_can_be_generated_and_verified() {
  // create message
  let message = {
    let manifest = Manifest {
      files: Directory::new(),
      notes: Vec::new(),
    };

    let message = Message {
      fingerprint: manifest.fingerprint(),
      time: None,
    };

    message.serialize()
  };

  // create cert
  let (cert, _revocation) = CertBuilder::new()
    .add_userid("foo@bar")
    .add_signing_subkey()
    .generate()
    .unwrap();

  // create policy
  let policy = StandardPolicy::new();

  // get signing key
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

  // create signature packet
  let signature_packet = {
    let mut keypair = signing_key.key().clone().into_keypair().unwrap();

    let signature_packet = SignatureBuilder::new(SignatureType::Binary)
      .set_hash_algo(HashAlgorithm::SHA512)
      .sign_message(&mut keypair, message.as_bytes())
      .unwrap();

    signature_packet
      .clone()
      .verify_message(signing_key.key(), message.as_bytes())
      .unwrap();

    signature_packet
  };

  // generate signature
  let signature = {
    let mpi::Signature::EdDSA { r, s } = signature_packet.mpis() else {
      panic!("expected EdDSA signature");
    };

    let r_bytes = r.value_padded(32).unwrap();
    let s_bytes = s.value_padded(32).unwrap();

    let mut sig_bytes = [0u8; 64];
    sig_bytes[..32].copy_from_slice(&r_bytes);
    sig_bytes[32..].copy_from_slice(&s_bytes);

    ed25519_dalek::Signature::from_bytes(&sig_bytes)
  };

  // extract public key
  let public_key = {
    let mpi::PublicKey::EdDSA { q, .. } = signing_key.key().mpis() else {
      panic!("expected EdDSA public key");
    };

    let (public_key_bytes, _) = q.decode_point(&Curve::Ed25519).unwrap();

    PublicKey::from_bytes(public_key_bytes.try_into().unwrap()).unwrap()
  };

  // check signature
  {
    let hashed_area = signature_packet.hashed_area().to_vec().unwrap();
    let hashed_area_len = hashed_area.len();

    let mut header = [0u8; 6];
    header[0] = 4;
    header[1] = u8::from(signature_packet.typ());
    header[2] = u8::from(signature_packet.pk_algo());
    header[3] = u8::from(signature_packet.hash_algo());
    #[allow(clippy::cast_possible_truncation)]
    header[4..6].copy_from_slice(&(hashed_area_len as u16).to_be_bytes());

    let mut trailer = [0u8; 6];
    trailer[0] = 4;
    trailer[1] = 0xff;
    #[allow(clippy::cast_possible_truncation)]
    let len = (header.len() + hashed_area_len) as u32;
    trailer[2..6].copy_from_slice(&len.to_be_bytes());

    let mut hasher = Sha512::new();
    hasher.update(message.as_bytes());
    hasher.update(header);
    hasher.update(&hashed_area);
    hasher.update(trailer);
    let digest = hasher.finalize();

    let verifying_key = public_key.inner();
    verifying_key.verify_strict(&digest, &signature).unwrap();
  }

  // extract and verify public key
  {
    let packet::key::SecretKeyMaterial::Unencrypted(secret_key_material) =
      signing_key.key().optional_secret().unwrap()
    else {
      panic!("expected unencrypted secret key");
    };

    let mpi::SecretKeyMaterial::EdDSA { scalar } = secret_key_material.map(Clone::clone) else {
      panic!("expected EdDSA secret key");
    };

    let private_key = PrivateKey::from_bytes(scalar.value().try_into().unwrap());

    assert_eq!(private_key.public_key(), public_key);
  }

  // create filepack signature
  let signature = Signature::new(
    SignatureScheme::Pgp {
      hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
    },
    signature,
  );

  // verify signature
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
