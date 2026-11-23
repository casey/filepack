use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct VideoMetadata {
  pub(crate) duration: u64,
  pub(crate) title: Option<Line>,
  pub(crate) tracks: Vec<Track>,
}
