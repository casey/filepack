use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ArchiveError {
  #[snafu(display("invalid file signature"))]
  FileSignature { backtrace: Option<Backtrace> },
  #[snafu(display("I/O error"))]
  FilesystemIo {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("archive truncated"))]
  Truncated { backtrace: Option<Backtrace> },
}
