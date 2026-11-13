use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum MountError {
  #[snafu(display("invalid mount name `{name}`"))]
  Name { name: String },
  #[snafu(transparent)]
  Package { source: PackageIdentifierError },
}
