use {
  super::*,
  ed25519_dalek::{Signer, pkcs8::EncodePrivateKey},
  rcgen::{CertificateParams, PKCS_ED25519, PublicKeyData, SignatureAlgorithm, SigningKey},
  rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
  tokio::runtime::Runtime,
  web_transport_quinn::ServerBuilder,
};

// todo:
// - get rid of multi-threading, make test friendly
// - probably want sans-I/O abstraction
//   - no web transport
//   - no storage
// - auth connection
// - require encrypted clienthello
// - understand encryption/replay/auth of datagrams
// - flow
//   - client tells server pin package
//   - client notifies server that it has package root
//   - server notifies client that it is interested in package
//   - client unchokes server
//   - server starts requesting files
//   - as server gets directory entries, it notifies client that it wants them
//   - server notifies client that it wants directories and metadata first
//   - server starts registering interest in files
//   - transfers done
//   - maybe server notifies client that it has everything
//   - client disconnects
//
// - after pinning package, server or client notifies that it's interested in package?
//   - server has reason to believe client has it
//   - client knows that server wants it
//
//
// - transfer messages
//   - handshake (via ALPN, possible w/WebTorrent?)
//   - keep alive (via quic ping)
//   - pin hash + directory flag -> file pinned
//   - have hash
//   - want hash
//   - want some pieces
//   - don't want any pieces
//   - unchoke (brake? slot? pause? open/close?)
//   - choke
//   - ask for hashes
//   - ask for piece
//   - cancel ask
//   - piece bitmap
//   - piece
// - dht messages
// - control messages
// - requests
//   - search
//   - list

#[derive(Parser)]
pub(crate) struct Node {}

impl Node {
  pub(crate) fn run(self) -> Result {
    let private_key = PrivateKey::generate();

    let (certificate, private_key) = certificate(&private_key).unwrap();

    let address = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8443, 0, 0);

    let mut server = ServerBuilder::new()
      .with_addr(address.into())
      .with_certificate(certificate, private_key)
      .unwrap();

    let runtime = Runtime::new().unwrap();

    runtime.block_on(async {
      while let Some(request) = server.accept().await {
        let session = request.ok().await.unwrap();
      }
    });

    Ok(())
  }
}

struct EdSigner<'a> {
  inner: &'a PrivateKey,
  public: [u8; PublicKey::LEN],
}

impl<'a> EdSigner<'a> {
  fn new(inner: &'a PrivateKey) -> Self {
    Self {
      public: inner.public_key().inner().to_bytes(),
      inner,
    }
  }
}

impl PublicKeyData for EdSigner<'_> {
  fn algorithm(&self) -> &'static SignatureAlgorithm {
    &PKCS_ED25519
  }

  fn der_bytes(&self) -> &[u8] {
    &self.public
  }
}

impl SigningKey for EdSigner<'_> {
  fn sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, rcgen::Error> {
    Ok(self.inner.inner_secret().sign(msg).to_bytes().to_vec())
  }
}

fn certificate(
  key: &PrivateKey,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), rcgen::Error> {
  let signer = EdSigner::new(key);

  let cert = CertificateParams::new(["localhost".to_string()])?.self_signed(&signer)?;

  let pkcs8 = key.inner_secret().to_pkcs8_der().unwrap();

  Ok((
    vec![cert.into()],
    PrivatePkcs8KeyDer::from(pkcs8.as_bytes().to_vec()).into(),
  ))
}
