use {
  super::*,
  axum::extract::{FromRequest, Request},
};

pub(crate) struct DecoRequest<T, const LIMIT: usize>(pub(crate) T);

impl<T: DecodeOwned, S: Send + Sync, const LIMIT: usize> FromRequest<S> for DecoRequest<T, LIMIT> {
  type Rejection = ServerError;

  async fn from_request(request: Request, _state: &S) -> ServerResult<Self> {
    let bytes = axum::body::to_bytes(request.into_body(), LIMIT)
      .await
      .context(server_error::DecoBody)?;

    Ok(Self(
      T::decode_from_slice(&bytes).context(server_error::DecoDecode)?,
    ))
  }
}

impl<T: Encode, const LIMIT: usize> Describe for DecoRequest<T, LIMIT> {
  fn describe(endpoint: &mut reference::Endpoint, registry: &mut deco::Registry) {
    let name = T::deco_schema(registry);

    let schema = registry.remove(&name).unwrap();

    endpoint.request = Some(reference::Request::Deco {
      limit: LIMIT.into_u64(),
      schema,
    });
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tokio::runtime::Runtime};

  #[test]
  fn body_exceeding_limit_is_rejected() {
    let request = Request::builder().body(Body::from(vec![0; 5])).unwrap();

    let result = Runtime::new()
      .unwrap()
      .block_on(DecoRequest::<Vec<u8>, 4>::from_request(request, &()));

    assert_matches!(
      result.map(|DecoRequest(value)| value),
      Err(ServerError::DecoBody { .. }),
    );
  }
}
