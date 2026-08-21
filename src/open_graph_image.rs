use super::*;

#[derive(Debug, PartialEq)]
pub struct OpenGraphImage {
  pub(crate) dimensions: Dimensions,
  pub(crate) path: String,
}

impl OpenGraphImage {
  pub(crate) fn artwork(metadata: &Metadata, fingerprint: Fingerprint) -> Option<Self> {
    let artwork = metadata.artwork.as_ref()?;
    Some(Self {
      dimensions: artwork.oriented_dimensions(),
      path: format!("artwork/{fingerprint}"),
    })
  }
}
