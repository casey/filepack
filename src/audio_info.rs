#[derive(Debug, PartialEq)]
pub(crate) struct AudioInfo {
  pub(crate) channels: u64,
  pub(crate) sample_bits: Option<u64>,
  pub(crate) sample_rate: u64,
  pub(crate) samples: u64,
}
