use super::*;

pub(crate) trait ReqwestResponseExt {
  fn cbor<T: Decode>(self) -> Result<T>;
}

impl ReqwestResponseExt for reqwest::blocking::Response {
  fn cbor<T: Decode>(self) -> Result<T> {
    let url = self.url().clone();

    let bytes = self
      .bytes()
      .with_context(|_| error::ResponseBody { url: url.clone() })?;

    T::decode_from_slice(&bytes).context(error::DecodeResponse { url })
  }
}
