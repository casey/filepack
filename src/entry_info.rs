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
  /// A file.
  #[n(0)]
  File,
  /// A directory.
  #[n(1)]
  Directory {
    /// Totals of the directory's contents.
    #[n(1)]
    totals: Totals,
  },
}
