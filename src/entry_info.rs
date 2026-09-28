use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Decode, Encode, EnumDiscriminants, PartialEq)]
#[strum_discriminants(
  allow(clippy::arbitrary_source_item_ordering),
  derive(Display),
  name(EntryType),
  strum(serialize_all = "kebab-case")
)]
pub enum EntryInfo {
  #[n(0)]
  File,
  #[n(1)]
  Directory {
    #[n(1)]
    totals: Totals,
  },
}
