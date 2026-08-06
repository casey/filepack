use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct AudioMetadata {
  pub(crate) album: Text,
  pub(crate) artist: Text,
  pub(crate) disc: u64,
  pub(crate) discs: u64,
  pub(crate) info: AudioInfo,
  pub(crate) title: Text,
  pub(crate) track: u64,
  pub(crate) tracks: u64,
}
