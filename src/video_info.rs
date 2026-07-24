use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct VideoInfo {
  pub(crate) duration: u64,
  pub(crate) tracks: Vec<Track>,
}
