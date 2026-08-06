use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct AudioMetadata {
  pub(crate) album: Text,
  pub(crate) artist: Text,
  pub(crate) channels: u64,
  pub(crate) disc: u64,
  pub(crate) discs: u64,
  pub(crate) sample_bits: Option<u64>,
  pub(crate) sample_rate: u64,
  pub(crate) samples: u64,
  pub(crate) title: Text,
  pub(crate) track: u64,
  pub(crate) tracks: u64,
}
