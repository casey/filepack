use {super::*, std::process::Command};

#[test]
fn ssh_signatures_can_be_verified() {
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

  let key_path = path.join("id_ed25519");

  let output = Command::new("ssh-keygen")
    .args(["-t", "ed25519"])
    .args(["-f", key_path.as_str()])
    .args(["-N", ""])
    .arg("-q")
    .output()
    .unwrap();
  assert!(output.status.success());

  let message_path = path.join("message");
  filesystem::write(&message_path, message.filepack_signed_data()).unwrap();

  let status = Command::new("ssh-keygen")
    .args(["-Y", "sign"])
    .args(["-f", key_path.as_str()])
    .args(["-n", "filepack", message_path.as_str()])
    .output()
    .unwrap();
  assert!(output.status.success());

  let public_key = {
    let public_key = filesystem::read_to_string(key_path.with_extension("pub"))
      .unwrap()
      .trim()
      .parse::<ssh_key::PublicKey>()
      .unwrap();

    let ssh_key::public::KeyData::Ed25519(public_key) = public_key.key_data() else {
      panic!("expected ed25519");
    };

    PublicKey::from_bytes(public_key.0)
  };

  {
    let pem = filesystem::read_to_string(&key_path).unwrap();
    let private_key = ssh_key::PrivateKey::from_openssh(&pem).unwrap();

    let ssh_key::private::KeypairData::Ed25519(keypair) = private_key.key_data() else {
      panic!("expected ed25519");
    };

    assert_eq!(PublicKey::from_bytes(*keypair.public.as_ref()), public_key);

    let private_key = PrivateKey::from_bytes(keypair.private.to_bytes());

    assert_eq!(private_key.public_key(), public_key,);
  };

  let signature = {
    let signature = filesystem::read_to_string(path.join("message.sig"))
      .unwrap()
      .parse::<ssh_key::SshSig>()
      .unwrap();
    Signature::new(
      SignatureScheme::Ssh,
      ed25519_dalek::Signature::from_bytes(&signature.signature_bytes().try_into().unwrap()),
    )
  };

  signature.verify(&message, public_key).unwrap();
}
