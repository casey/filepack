use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum Version {
  #[default]
  #[n(0)]
  Zero,
}
