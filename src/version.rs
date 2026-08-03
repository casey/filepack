use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub(crate) enum Version {
  #[default]
  #[n(0)]
  Zero,
}
