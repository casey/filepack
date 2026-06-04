use {
  super::*,
  axum::extract::{FromRequest, Request},
};

const REQUEST_LIMIT: usize = 16 * 1024 * 1024;

pub(crate) struct Cbor<T>(pub(crate) T);

impl<T: Decode, S: Send + Sync> FromRequest<S> for Cbor<T> {
  type Rejection = ServerError;

  async fn from_request(request: Request, _state: &S) -> ServerResult<Self> {
    let bytes = axum::body::to_bytes(request.into_body(), REQUEST_LIMIT)
      .await
      .context(server_error::CborBody)?;

    Ok(Self(
      T::decode_from_slice(&bytes).context(server_error::CborDecode)?,
    ))
  }
}
