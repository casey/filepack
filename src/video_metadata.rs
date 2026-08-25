use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct VideoMetadata {
  pub(crate) duration: u64,
  pub(crate) title: Option<Text>,
  pub(crate) tracks: Vec<Track>,
}
