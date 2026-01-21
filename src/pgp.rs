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
  std::process::Command,
};

#[test]
fn gpg_v4_signatures_can_be_verified() {
  if cfg!(not(unix)) {
    return;
  }

  // create tempdir
  let tempdir = tempdir();
  let path = decode_path(tempdir.path()).unwrap();

  // create gpg homedir
  let home = path.join("gnupg");
  {
    fs::create_dir(&home).unwrap();

    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      fs::set_permissions(&home, Permissions::from_mode(0o700)).unwrap();
    }
  }

  // generate keypair
  {
    let output = Command::new("gpg")
      .args(["--homedir", home.as_str()])
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
  }

  // create and sign message
  let signature_path = path.join("message.sig");
  let message = {
    let manifest = Manifest {
      files: Directory::new(),
      notes: Vec::new(),
    };

    let message = Message {
      fingerprint: manifest.fingerprint(),
      time: None,
    }
    .serialize();

    let message_path = path.join("message");
    filesystem::write(&message_path, message.bytes()).unwrap();

    let output = Command::new("gpg")
      .args(["--homedir", home.as_str()])
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

    message
  };

  // extract public key
  let public_key = {
    let policy = StandardPolicy::new();

    let output = Command::new("gpg")
      .args(["--homedir", home.as_str()])
      .args(["--export", "foo@bar"])
      .output()
      .unwrap();

    assert!(
      output.status.success(),
      "gpg export failed: {}",
      String::from_utf8_lossy(&output.stderr)
    );

    let cert = Cert::from_bytes(&output.stdout).unwrap();

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
    PublicKey::from_bytes(public_key_bytes.try_into().unwrap())
  };

  // extract and verify private key
  {
    let policy = StandardPolicy::new();

    let output = Command::new("gpg")
      .args(["--homedir", home.as_str()])
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

    let mpi::SecretKeyMaterial::EdDSA { scalar } = secret_key_material.map(Clone::clone) else {
      panic!("expected EdDSA secret key");
    };

    let private_key = PrivateKey::from_bytes(scalar.value().try_into().unwrap());

    assert_eq!(private_key.public_key(), public_key);
  }

  // extract signature
  let signature = {
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

    Signature::new(
      SignatureScheme::Pgp {
        hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
      },
      ed25519_dalek::Signature::from_bytes(&sig_bytes),
    )
  };

  // verify signature
  signature.verify(&message, public_key).unwrap();
}

#[test]
fn pgp_v4_signatures_can_be_verified() {
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
      .sign_message(&mut keypair, message.bytes())
      .unwrap();

    signature_packet
      .clone()
      .verify_message(signing_key.key(), message.bytes())
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

    PublicKey::from_bytes(public_key_bytes.try_into().unwrap())
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
    hasher.update(message.bytes());
    hasher.update(header);
    hasher.update(&hashed_area);
    hasher.update(trailer);
    let digest = hasher.finalize();

    let verifying_key = public_key.inner();
    verifying_key.verify_strict(&digest, &signature).unwrap();
  }

  // extract and verify public key
  let private_key = {
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

    private_key
  };

  // create filepack signature
  let signature = Signature::new(
    SignatureScheme::Pgp {
      hashed_area: signature_packet.hashed_area().to_vec().unwrap(),
    },
    signature,
  );

  // verify signature
  signature.verify(&message, public_key).unwrap();

  eprintln!("PGP_PRIVATE_KEY: {}", private_key.display_secret());
  eprintln!("PGP_PUBLIC_KEY: {public_key}");
  eprintln!("PGP_SIGNATURE: {signature}");
}
