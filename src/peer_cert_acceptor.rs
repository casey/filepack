use {
  super::*,
  axum::extract::Extension,
  axum_server::{accept::Accept, tls_rustls::RustlsAcceptor},
  std::{future::Future, pin::Pin},
  tokio::io::{AsyncRead, AsyncWrite},
  tokio_rustls::server::TlsStream,
  tower::Layer,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct PeerPublicKey(pub(crate) Option<PublicKey>);

#[derive(Clone)]
pub(crate) struct PeerCertAcceptor {
  pub(crate) inner: RustlsAcceptor,
}

impl<I, S> Accept<I, S> for PeerCertAcceptor
where
  I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
  S: Send + 'static,
{
  type Future = Pin<Box<dyn Future<Output = io::Result<(Self::Stream, Self::Service)>> + Send>>;
  type Service = <Extension<PeerPublicKey> as Layer<S>>::Service;
  type Stream = TlsStream<I>;

  fn accept(&self, stream: I, service: S) -> Self::Future {
    let inner = self.inner.clone();
    Box::pin(async move {
      let (tls, service) = inner.accept(stream, service).await?;
      let peer = tls
        .get_ref()
        .1
        .peer_certificates()
        .and_then(|c| c.first())
        .map(crate::client_cert_verifier::extract_ed25519_pubkey);
      let service = Extension(PeerPublicKey(peer)).layer(service);
      Ok((tls, service))
    })
  }
}
